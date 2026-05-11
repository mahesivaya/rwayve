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
  COMPOSE=(docker compose -f "$COMPOSE_FILE")
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE=(docker-compose -f "$COMPOSE_FILE")
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

log "building + starting stack"
"${COMPOSE[@]}" up -d --build

log "waiting for backend to respond at $API_HOST (up to ${HEALTH_TIMEOUT_S}s)"
deadline=$(( $(date +%s) + HEALTH_TIMEOUT_S ))
while :; do
  status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 "$API_HOST/api/me" || echo "000")
  if [[ "$status" == "401" ]]; then
    ok "backend up (got expected 401 from /api/me)"
    break
  fi
  if (( $(date +%s) >= deadline )); then
    log "backend logs (last 100 lines):"
    "${COMPOSE[@]}" logs --tail=100 backend || true
    fail "backend did not become healthy within ${HEALTH_TIMEOUT_S}s (last status=$status)"
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

# Auth gating
assert_status 401 "GET /api/me without token returns 401" "$API_HOST/api/me"

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
