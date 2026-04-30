#!/usr/bin/env bash
set -euo pipefail

GETH_RPC_URL="${GETH_RPC_URL:-http://localhost:18545}"
CONNECT_TIMEOUT_SECONDS="${CONNECT_TIMEOUT_SECONDS:-2}"
MAX_TIME_SECONDS="${MAX_TIME_SECONDS:-10}"

echo "Geth RPC probe target: ${GETH_RPC_URL}"
echo

rpc() {
  local method="$1"
  local params="${2:-[]}"
  local tmp_body
  local status
  tmp_body="$(mktemp)"

  if ! status="$(
    curl -sS -o "${tmp_body}" -w "%{http_code}" \
      --connect-timeout "${CONNECT_TIMEOUT_SECONDS}" \
      --max-time "${MAX_TIME_SECONDS}" \
      -H "Content-Type: application/json" \
      --data "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"${method}\",\"params\":${params}}" \
      "${GETH_RPC_URL}"
  )"; then
    echo "--- ${method}"
    echo "error: Geth RPC request failed or timed out"
    sed -n '1,20p' "${tmp_body}"
    rm -f "${tmp_body}"
    return 1
  fi

  echo "--- ${method}"
  echo "status: ${status}"
  sed -n '1,20p' "${tmp_body}"
  echo

  if [[ "${status}" != 2* ]]; then
    rm -f "${tmp_body}"
    echo "error: expected 2xx response from ${method}" >&2
    return 1
  fi

  if grep -q '"error"' "${tmp_body}"; then
    rm -f "${tmp_body}"
    echo "error: JSON-RPC error returned by ${method}" >&2
    return 1
  fi

  if [ "${method}" = "eth_accounts" ]; then
    python3 - "$tmp_body" <<'PY'
import json
import sys
path = sys.argv[1]
with open(path, "r", encoding="utf-8") as handle:
    payload = json.load(handle)
accounts = payload.get("result") or []
if not accounts:
    raise SystemExit("error: eth_accounts returned no unlocked sender account")
PY
  fi

  rm -f "${tmp_body}"
}

rpc "web3_clientVersion"
rpc "eth_chainId"
rpc "eth_accounts"
