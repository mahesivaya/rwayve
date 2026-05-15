#!/usr/bin/env bash
# Pre-deployment gate for rwayve. Run this BEFORE building/deploying an
# environment — it validates env files + secrets and runs the backend and
# frontend verification suites.
#
#   ./scripts/preflight.sh [development|production]   # default: production
#   SKIP_BUILD=1 ./scripts/preflight.sh               # env/secret checks only
#
# Exits non-zero if any check fails, so it can gate a deploy pipeline.

set -uo pipefail

ENV_NAME="${1:-production}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[0;33m'; BLUE='\033[0;34m'; NC='\033[0m'
FAILURES=0

log()  { printf "${BLUE}[preflight]${NC} %s\n" "$*"; }
ok()   { printf "  ${GREEN}✓${NC} %s\n" "$*"; }
warn() { printf "  ${YELLOW}!${NC} %s\n" "$*"; }
bad()  { printf "  ${RED}✗${NC} %s\n" "$*"; FAILURES=$((FAILURES + 1)); }

case "$ENV_NAME" in
  development|production) ;;
  *) echo "usage: $0 [development|production]" >&2; exit 2 ;;
esac

# Read a KEY=value line from an env file: trims surrounding whitespace and any
# ` # inline comment` (dotenv-style, comment must be preceded by whitespace).
get_var() {
  grep -E "^$1=" "$2" 2>/dev/null | head -1 | cut -d= -f2- \
    | sed -e 's/[[:space:]]\{1,\}#.*$//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//'
}

log "environment: $ENV_NAME"

# ── 1. Env files present ──────────────────────────────────────────────────
log "checking env files"
BACKEND_ENV="backend/.env.${ENV_NAME}"
for f in ".env.${ENV_NAME}" "$BACKEND_ENV" "frontend/.env.${ENV_NAME}" "infra/.env.${ENV_NAME}"; do
  if [[ -f "$f" ]]; then ok "$f exists"; else bad "$f missing (copy from ${f}.example)"; fi
done

# ── 2. Secret strength (backend env) ──────────────────────────────────────
if [[ -f "$BACKEND_ENV" ]]; then
  log "validating secrets in $BACKEND_ENV"

  jwt=$(get_var JWT_SECRET "$BACKEND_ENV")
  if   [[ -z "$jwt" ]];                       then bad "JWT_SECRET is empty"
  elif [[ "$jwt" == "secret" || "$jwt" == changeme* ]]; then bad "JWT_SECRET is a placeholder"
  elif (( ${#jwt} < 32 ));                    then bad "JWT_SECRET too short (${#jwt} chars; want >= 32)"
  else ok "JWT_SECRET looks strong (${#jwt} chars)"; fi

  aes=$(get_var AES_KEY "$BACKEND_ENV")
  if   [[ -z "$aes" ]]; then bad "AES_KEY is empty"
  elif [[ "$aes" == "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f" ]]; then
    if [[ "$ENV_NAME" == "production" ]]; then
      bad "AES_KEY is the dev placeholder — generate one: openssl rand -hex 32"
    else
      warn "AES_KEY is the shared dev placeholder (acceptable for dev, never for prod)"
    fi
  elif [[ ! "$aes" =~ ^[0-9a-fA-F]{64}$ ]]; then
    bad "AES_KEY must be 64 hex chars (got ${#aes})"
  else ok "AES_KEY is a valid Hex64 key"; fi

  for v in DATABASE_URL FRONTEND_URL; do
    if [[ -z "$(get_var "$v" "$BACKEND_ENV")" ]]; then bad "$v is empty"; else ok "$v is set"; fi
  done

  if [[ "$ENV_NAME" == "production" ]]; then
    fe=$(get_var FRONTEND_URL "$BACKEND_ENV")
    if [[ "$fe" == https://* ]]; then ok "FRONTEND_URL uses https"; else warn "FRONTEND_URL is not https ($fe)"; fi
  fi
fi

# ── 3. No secrets tracked in git ──────────────────────────────────────────
# Only non-empty files count — an empty tracked `.env.prod` leaks nothing.
log "checking git for committed secrets"
leaked=""
while IFS= read -r f; do
  [[ -n "$f" && -s "$f" ]] && leaked+="$f "
done < <(git ls-files 2>/dev/null \
  | grep -E '(^|/)\.env($|\.)|client_secret\.json$|\.pem$' | grep -vE '\.example$' || true)
if [[ -n "$leaked" ]]; then
  bad "non-empty secret-like files are tracked in git:"; printf '      %s\n' $leaked
else
  ok "no env files / keys / client_secret.json with content tracked"
fi

# ── 4. Schema file present ────────────────────────────────────────────────
if [[ -f infra/postgres/init.sql ]]; then ok "infra/postgres/init.sql present"
else bad "infra/postgres/init.sql missing"; fi

# ── 5. Backend verification ───────────────────────────────────────────────
if [[ "${SKIP_BUILD:-0}" == "1" ]]; then
  warn "SKIP_BUILD=1 — skipping backend + frontend build/test steps"
else
  if command -v cargo >/dev/null 2>&1; then
    log "backend: fmt / clippy / test"
    ( cd backend && cargo fmt --check )                         && ok "cargo fmt"    || bad "cargo fmt --check failed"
    ( cd backend && cargo clippy --quiet -- -D warnings )       && ok "cargo clippy" || bad "cargo clippy failed"
    ( cd backend && cargo test --quiet -- --test-threads=1 )    && ok "cargo test"   || bad "cargo test failed"
  else
    warn "cargo not found — skipping backend checks"
  fi

  # ── 6. Frontend verification ────────────────────────────────────────────
  if command -v npm >/dev/null 2>&1; then
    log "frontend: build / test"
    ( cd frontend && npm run --silent build )  && ok "vite build (tsc)" || bad "frontend build failed"
    ( cd frontend && npm test --silent )       && ok "vitest"           || bad "frontend tests failed"
  else
    warn "npm not found — skipping frontend checks"
  fi
fi

# ── Summary ───────────────────────────────────────────────────────────────
echo
if (( FAILURES == 0 )); then
  printf "${GREEN}preflight passed — safe to deploy %s${NC}\n" "$ENV_NAME"
  exit 0
else
  printf "${RED}preflight FAILED — %d check(s) need attention before deploying${NC}\n" "$FAILURES"
  exit 1
fi
