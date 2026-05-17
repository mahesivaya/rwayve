#!/usr/bin/env bash
# Docker smoke test for rwayve.
#
# 1. Brings up the full compose stack.
# 2. Waits for postgres + backend to report healthy.
# 3. Probes a handful of HTTP endpoints to verify they respond as expected:
#       GET  /api/me                            -> 401 (no token)
#       POST /api/login (bad creds)             -> 401
#       POST /api/forgot-password (bad email)   -> 200 (generic response)
#       GET  /                                  -> 200 (frontend served by nginx)
# 4. Tears the stack down (unless KEEP_RUNNING=1).
#
# Usage:
#   ./scripts/smoke.sh
#   KEEP_RUNNING=1 ./scripts/smoke.sh   # leave compose up for inspection
#   COMPOSE_FILE=infra/docker-compose.yml ./scripts/smoke.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-$REPO_ROOT/infra/docker-compose.yml}"
ENV_FILE="${ENV_FILE:-$REPO_ROOT/.env}"

API_HOST="${API_HOST:-http://localhost:8080}"
WEB_HOST="${WEB_HOST:-http://localhost:80}"
HEALTH_TIMEOUT_S="${HEALTH_TIMEOUT_S:-90}"

BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

log()  { printf "${BLUE}[smoke]${NC} %s\n" "$*"; }
ok()   { printf "${GREEN}  ✓${NC} %s\n" "$*"; }
fail() { printf "${RED}  ✗${NC} %s\n" "$*" >&2; exit 1; }

# Pick the right compose subcommand for the host.
if docker compose version >/dev/null 2>&1; then
  COMPOSE=(docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE")
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE=(docker-compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE")
else
  fail "neither 'docker compose' nor 'docker-compose' available"
fi

cleanup() {
  if [[ "${KEEP_RUNNING:-}" == "1" ]]; then
    log "KEEP_RUNNING=1 set — leaving stack up. Tear down with:"
    log "  ${COMPOSE[*]} down -v"
    return
  fi
  log "tearing down stack"
  "${COMPOSE[@]}" down -v --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

if [[ -f "$ENV_FILE" ]]; then
  log "using env file: $ENV_FILE"
fi

log "preparing environment files from templates"
# Docker Compose hard-fails if an 'env_file' specified in the YAML is missing.
# In CI environments, these files are absent. We populate them from .example templates.
for f in ".env" ".env.development" "backend/.env.development" "frontend/.env.development" "infra/.env.development"; do
  if [[ ! -f "$REPO_ROOT/$f" && -f "$REPO_ROOT/$f.example" ]]; then
    log "creating $f"
    cp "$REPO_ROOT/$f.example" "$REPO_ROOT/$f"
  fi
done

# Create a dummy client_secret.json if missing to prevent Docker from mounting a directory
if [[ ! -f "$REPO_ROOT/client_secret.json" ]]; then
  log "creating dummy client_secret.json"
  echo "{}" > "$REPO_ROOT/client_secret.json"
fi

log "building + starting stack"
"${COMPOSE[@]}" up -d --build

log "waiting for backend + database to be ready at $API_HOST (up to ${HEALTH_TIMEOUT_S}s)"
deadline=$(( $(date +%s) + HEALTH_TIMEOUT_S ))
while :; do
  status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 "$API_HOST/api/ready" || echo "000")
  if [[ "$status" == "200" ]]; then
    ok "backend + database ready"
    break
  fi
  if (( $(date +%s) >= deadline )); then
    log "backend logs (last 100 lines):"
    "${COMPOSE[@]}" logs --tail=100 backend || true
    fail "backend/database did not become ready within ${HEALTH_TIMEOUT_S}s (last status=$status)"
  fi
  sleep 2
done

log "waiting for frontend through nginx at $WEB_HOST (up to ${HEALTH_TIMEOUT_S}s)"
deadline=$(( $(date +%s) + HEALTH_TIMEOUT_S ))
while :; do
  status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 "$WEB_HOST/" || echo "000")
  if [[ "$status" =~ ^(200|301|302)$ ]]; then
    ok "frontend up through nginx (got $status from /)"
    break
  fi
  if (( $(date +%s) >= deadline )); then
    log "frontend logs (last 100 lines):"
    "${COMPOSE[@]}" logs --tail=100 frontend || true
    log "nginx logs (last 100 lines):"
    "${COMPOSE[@]}" logs --tail=100 nginx || true
    fail "frontend did not become reachable within ${HEALTH_TIMEOUT_S}s (last status=$status)"
  fi
  sleep 2
done

log "running smoke checks"

assert_status() {
  local expected="$1"; local desc="$2"; shift 2
  local got
  got=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$@" || echo "000")
  if [[ "$got" == "$expected" ]]; then
    ok "$desc (got $got)"
  else
    fail "$desc: expected $expected, got $got"
  fi
}

# Health + readiness probes
assert_status 200 "GET /api/health returns 200 (liveness)" "$API_HOST/api/health"
assert_status 200 "GET /api/ready returns 200 (Postgres + Redis reachable)" "$API_HOST/api/ready"

# Auth gating
assert_status 401 "GET /api/me without token returns 401" "$API_HOST/api/me"

# Security Probes: Administrative routes
assert_status 401 "GET /api/admin/organizations without token returns 401" "$API_HOST/api/admin/organizations"
assert_status 401 "GET /api/users/all without token returns 401" "$API_HOST/api/users/all"

# Security Probes: Mailbox & Files
assert_status 401 "GET /api/emails without token returns 401" "$API_HOST/api/emails"
assert_status 401 "GET /api/files without token returns 401" "$API_HOST/api/files"
assert_status 401 "GET /api/email-attachments/1/download (Outlook/Gmail) requires auth" "$API_HOST/api/email-attachments/1/download"
assert_status 401 "POST /api/outlook/connect-url (Outlook) requires auth" -X POST "$API_HOST/api/outlook/connect-url"

# Login with invalid creds
assert_status 401 "POST /api/login with bad creds returns 401" \
  -X POST -H "Content-Type: application/json" \
  --data '{"email":"nobody@nowhere.test","password":"x"}' \
  "$API_HOST/api/login"

# Forgot password is intentionally generic — must NOT 5xx
assert_status 200 "POST /api/forgot-password returns 200 even for unknown email" \
  -X POST -H "Content-Type: application/json" \
  --data '{"email":"nobody@nowhere.test"}' \
  "$API_HOST/api/forgot-password"

# Reset password with a bogus token must be a clean 400, not 500
assert_status 400 "POST /api/reset-password with bogus token returns 400" \
  -X POST -H "Content-Type: application/json" \
  --data '{"token":"no-such-token","new_password":"longenough"}' \
  "$API_HOST/api/reset-password"

# Outlook OAuth Probes
log "probing Outlook OAuth routes"
# Probing public login redirects. This will return 302 if configured, or 500 if env vars are missing.
# Either way, getting a response verifies the route is registered and responding.
outlook_status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$API_HOST/outlook/login?mode=signup" || echo "000")
if [[ "$outlook_status" =~ ^(302|500)$ ]]; then
  ok "GET /outlook/login?mode=signup is reachable (got $outlook_status)"
else
  fail "GET /outlook/login?mode=signup: expected 302 or 500, got $outlook_status"
fi

# Frontend through nginx
if curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$WEB_HOST/" | grep -qE '^(200|301|302)$'; then
  ok "GET $WEB_HOST/ serves frontend"
else
  log "frontend logs (last 50 lines):"
  "${COMPOSE[@]}" logs --tail=50 frontend || true
  log "nginx logs (last 50 lines):"
  "${COMPOSE[@]}" logs --tail=50 nginx || true
  fail "frontend not reachable at $WEB_HOST/"
fi

log "smoke passed"
