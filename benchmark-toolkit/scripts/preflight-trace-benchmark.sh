#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

echo "[preflight] Running local trace benchmark gate"

cargo test web::sparql_validator::
cargo test --test bootstrap_sparql_api_contract_tests -- --nocapture
cargo test --test benchmark_query_contract_tests -- --nocapture
cargo test --test e2e_api_workflows test_sparql_query_processing_pipeline -- --nocapture
cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml fabric -- --nocapture
cargo test --manifest-path benchmark-toolkit/research-benchmarks/Cargo.toml --lib -- --nocapture

echo "[preflight] Local trace benchmark gate passed"
