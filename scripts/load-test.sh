#!/usr/bin/env bash
set -euo pipefail

# ── Config ──
API="http://127.0.0.1:3456/api/v1"
SDK_KEY="flk_sdk_zvKxFJkKOvegSshSAFDHjIaIcQZa5irc"
ENV_ID="e83b35e2-e5e6-4f20-bc0b-2842d2389d2b"  # development

CONCURRENCY=${1:-50}
REQUESTS=${2:-1000}

# Create temp POST body files
EVAL_BODY=$(mktemp)
BATCH_BODY=$(mktemp)
LOGIN_BODY=$(mktemp)
trap 'rm -f "$EVAL_BODY" "$BATCH_BODY" "$LOGIN_BODY"' EXIT

echo -n '{"flag_key":"dark-mode","context":{"key":"user-123","attributes":{"plan":"pro","country":"US"}}}' > "$EVAL_BODY"
echo -n '{"context":{"key":"user-456","attributes":{"plan":"free","country":"FR"}}}' > "$BATCH_BODY"
echo -n '{"email":"gaetan@juvin.net","password":"coucou42"}' > "$LOGIN_BODY"

echo "═══════════════════════════════════════════"
echo "  Flick Load Test"
echo "  Concurrency: $CONCURRENCY"
echo "  Requests:    $REQUESTS per endpoint"
echo "═══════════════════════════════════════════"
echo ""

# ── 1. Single flag evaluation ──
echo "▶ POST /evaluate (single flag: dark-mode)"
ab -n "$REQUESTS" -c "$CONCURRENCY" -q \
  -H "Authorization: Bearer $SDK_KEY" \
  -H "X-Environment-Id: $ENV_ID" \
  -T "application/json" \
  -p "$EVAL_BODY" \
  "$API/evaluate" 2>&1 | grep -E '(Requests per second|Time per request.*\(mean\)|Failed|Non-2xx|Complete requests|50%|95%|99%)'

echo ""

# ── 2. Batch evaluation ──
echo "▶ POST /evaluate/batch (all flags)"
ab -n "$REQUESTS" -c "$CONCURRENCY" -q \
  -H "Authorization: Bearer $SDK_KEY" \
  -H "X-Environment-Id: $ENV_ID" \
  -T "application/json" \
  -p "$BATCH_BODY" \
  "$API/evaluate/batch" 2>&1 | grep -E '(Requests per second|Time per request.*\(mean\)|Failed|Non-2xx|Complete requests|50%|95%|99%)'

echo ""

# ── 3. Config polling (what the SDK does) ──
echo "▶ GET /evaluate/config (SDK polling)"
ab -n "$REQUESTS" -c "$CONCURRENCY" -q \
  -H "Authorization: Bearer $SDK_KEY" \
  -H "X-Environment-Id: $ENV_ID" \
  "$API/evaluate/config" 2>&1 | grep -E '(Requests per second|Time per request.*\(mean\)|Failed|Non-2xx|Complete requests|50%|95%|99%)'

echo ""

# ── 4. Auth endpoint (bcrypt is intentionally slow) ──
echo "▶ POST /auth/login (bcrypt — expect slower)"
ab -n 100 -c 10 -q \
  -T "application/json" \
  -p "$LOGIN_BODY" \
  "$API/auth/login" 2>&1 | grep -E '(Requests per second|Time per request.*\(mean\)|Failed|Non-2xx|Complete requests|50%|95%|99%)'

echo ""

# ── 5. Health check baseline ──
echo "▶ GET /health (baseline)"
ab -n "$REQUESTS" -c "$CONCURRENCY" -q \
  "http://127.0.0.1:3456/health" 2>&1 | grep -E '(Requests per second|Time per request.*\(mean\)|Failed|Non-2xx|Complete requests|50%|95%|99%)'

echo ""
echo "═══════════════════════════════════════════"
echo "  Done."
echo "═══════════════════════════════════════════"
