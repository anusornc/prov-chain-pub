#!/usr/bin/env bash
set -euo pipefail

FABRIC_GATEWAY_URL="${FABRIC_GATEWAY_URL:-http://localhost:18800}"
CONNECT_TIMEOUT_SECONDS="${CONNECT_TIMEOUT_SECONDS:-2}"
MAX_TIME_SECONDS="${MAX_TIME_SECONDS:-10}"

echo "Fabric gateway probe target: $FABRIC_GATEWAY_URL"
echo

probe() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  local tmp_body
  local status
  tmp_body="$(mktemp)"

  if [[ -n "$body" ]]; then
    if ! status="$(
      curl -sS -o "$tmp_body" -w "%{http_code}" \
        --connect-timeout "$CONNECT_TIMEOUT_SECONDS" \
        --max-time "$MAX_TIME_SECONDS" \
        -X "$method" \
        -H "Content-Type: application/json" \
        --data "$body" \
        "$FABRIC_GATEWAY_URL$path"
    )"; then
      echo "--- $method $path"
      echo "error: Fabric gateway request failed or timed out"
      sed -n '1,20p' "$tmp_body"
      rm -f "$tmp_body"
      return 1
    fi
  else
    if ! status="$(
      curl -sS -o "$tmp_body" -w "%{http_code}" \
        --connect-timeout "$CONNECT_TIMEOUT_SECONDS" \
        --max-time "$MAX_TIME_SECONDS" \
        -X "$method" \
        "$FABRIC_GATEWAY_URL$path"
    )"; then
      echo "--- $method $path"
      echo "error: Fabric gateway request failed or timed out"
      sed -n '1,20p' "$tmp_body"
      rm -f "$tmp_body"
      return 1
    fi
  fi

  echo "--- $method $path"
  echo "status: $status"
  sed -n '1,20p' "$tmp_body"
  echo
  rm -f "$tmp_body"

  if [[ "$status" != 2* ]]; then
    echo "error: expected 2xx response from $method $path" >&2
    return 1
  fi
}

record='{
  "record_id": "record-001",
  "payload": {
    "entity_id": "BATCH001",
    "entity_type": "ProductBatch",
    "event_type": "Produced",
    "timestamp": "2026-04-24T00:00:00Z",
    "actor_id": "producer-001",
    "location_id": "site-001",
    "previous_record_ids": [],
    "attributes": {}
  },
  "policy": {
    "visibility": "public",
    "owner_org": "Org1MSP"
  }
}'

policy='{
  "record_id": "record-001",
  "actor_org": "Org1MSP",
  "action": "read"
}'

probe "GET" "/health"
probe "POST" "/ledger/records" "$record"
probe "POST" "/ledger/records/batch" "{\"records\":[$record]}"
probe "POST" "/policy/check" "$policy"
