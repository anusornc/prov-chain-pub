#!/bin/bash

# OWL2 Reasoner Coverage Analysis Script
# This script provides comprehensive test coverage analysis

set -e

echo "🔬 OWL2 Reasoner Test Coverage Analysis"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    print_status "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
    print_success "cargo-tarpaulin installed successfully"
fi

# Create coverage directory
COVERAGE_DIR="target/coverage"
mkdir -p "$COVERAGE_DIR"

print_status "Running comprehensive coverage analysis..."

# Run coverage analysis
if cargo tarpaulin \
    --out Html \
    --output-dir "$COVERAGE_DIR" \
    --verbose \
    --timeout 300 \
    --all-targets \
    --ignore-tests; then

    print_success "Coverage analysis completed successfully!"

    # Check if HTML report was generated
    if [ -f "$COVERAGE_DIR/tarpaulin.html" ]; then
        print_success "HTML coverage report generated: $COVERAGE_DIR/tarpaulin.html"

        # Try to open the report in default browser (macOS)
        if command -v open &> /dev/null; then
            print_status "Opening coverage report in browser..."
            open "$COVERAGE_DIR/tarpaulin.html"
        fi
    else
        print_warning "HTML report not found at expected location"
    fi

else
    print_error "Coverage analysis failed!"
    exit 1
fi

# Generate XML report for CI/CD integration
print_status "Generating XML coverage report for CI/CD integration..."
cargo tarpaulin \
    --out Xml \
    --output-dir "$COVERAGE_DIR" \
    --timeout 300 \
    --all-targets \
    --ignore-tests

if [ -f "$COVERAGE_DIR/tarpaulin.xml" ]; then
    print_success "XML coverage report generated: $COVERAGE_DIR/tarpaulin.xml"
else
    print_warning "XML report generation failed"
fi

print_status "Coverage analysis complete!"
echo "📊 Reports available in: $COVERAGE_DIR"
echo "   - HTML: $COVERAGE_DIR/tarpaulin.html"
echo "   - XML:  $COVERAGE_DIR/tarpaulin.xml"