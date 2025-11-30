#!/bin/bash

# OWL2 Reasoner System Validation Script
# This script validates the entire system functionality

set -e

echo "🔍 Starting OWL2 Reasoner System Validation"
echo "==============================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

echo "📦 Building project..."
cargo build

echo ""
echo "🧪 Running unit tests..."
cargo test --lib

echo ""
echo "📋 Running validation tests..."
cargo test --test validation_tests

echo ""
echo "🏃 Running example validation..."
echo "   Basic examples..."
cargo run --example family_ontology
cargo run --example biomedical_ontology

echo "   Benchmarking examples..."
cargo run --example benchmark_cli -- --help

echo "   Validation examples..."
cargo run --example complete_validation

echo ""
echo "✅ System validation completed successfully!"
echo "🎯 All tests passed - system is ready for use"