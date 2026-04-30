#!/usr/bin/env bash
set -euo pipefail

FLUREE_URL="${FLUREE_URL:-http://localhost:18090}"
FLUREE_LEDGER="${FLUREE_LEDGER:-provchain/benchmark}"

if [[ "$FLUREE_LEDGER" != */* ]]; then
  echo "FLUREE_LEDGER must be in <network>/<db> form, got: $FLUREE_LEDGER" >&2
  exit 1
fi

NETWORK="${FLUREE_LEDGER%%/*}"
DB="${FLUREE_LEDGER#*/}"

echo "Fluree probe target: $FLUREE_URL"
echo "Ledger: $NETWORK/$DB"
echo

probe() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  local tmp_body
  tmp_body="$(mktemp)"

  if [[ -n "$body" ]]; then
    status="$(
      curl -sS -o "$tmp_body" -w "%{http_code}" \
        -X "$method" \
        -H "Content-Type: application/json" \
        --data "$body" \
        "$FLUREE_URL$path"
    )"
  else
    status="$(
      curl -sS -o "$tmp_body" -w "%{http_code}" \
        -X "$method" \
        "$FLUREE_URL$path"
    )"
  fi

  echo "--- $method $path"
  echo "status: $status"
  sed -n '1,20p' "$tmp_body"
  echo
  rm -f "$tmp_body"
}

probe "GET" "/fdb/health"
probe "GET" "/index.html"
probe "POST" "/fluree/create" "{\"ledger\":\"$FLUREE_LEDGER\"}"
probe "POST" "/fluree/query" "{\"from\":\"$FLUREE_LEDGER\",\"select\":[\"?product\"],\"where\":{\"@id\":\"?product\",\"http://example.org/supplychain/batchId\":\"BATCH001\"}}"
