#!/bin/bash

# OWL2 Reasoner Benchmark Runner Script
# This script runs all benchmarks and generates comprehensive reports

set -e

echo "🚀 Starting OWL2 Reasoner Benchmark Suite"
echo "============================================"

# Usage/help
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  cat <<EOF
Usage: ./scripts/run_benchmarks.sh [--no-run]

Options:
  --no-run   Build all benches without executing them (fast sanity check)

This script:
  1) Builds the project (release)
  2) Optionally performs a benches compile-only sanity check
  3) Runs selected Criterion benches (unless --no-run)
  4) Runs the external Python benchmarking framework
  5) Generates reports
EOF
  exit 0
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Build the project first
echo "📦 Building project..."
cargo build --release

# Quick sanity: ensure benches compile (no execution)
echo "🧪 Sanity check: compiling benches (no run)..."
cargo bench --no-run

# If only compile is requested, stop here
if [[ "$1" == "--no-run" ]]; then
  echo "⏹️  --no-run specified: skipping bench execution and reports"
  echo "✅ Benches compiled successfully."
  exit 0
fi

echo ""
echo "🏃 Running Rust benchmarks..."
cargo bench --bench basic_benchmarks
cargo bench --bench performance_validation
cargo bench --bench scale_testing

echo ""
echo "🐍 Running Python benchmarking framework..."
cd benchmarking/framework
python benchmark_runner.py --all
cd ../..

echo ""
echo "📊 Generating comprehensive reports..."
python benchmarking/framework/generate_report.py

echo ""
echo "✅ Benchmark suite completed successfully!"
echo "📈 Results available in:"
echo "   - target/criterion/ (Rust benchmark results)"
echo "   - benchmarking/results/ (Python framework results)"
