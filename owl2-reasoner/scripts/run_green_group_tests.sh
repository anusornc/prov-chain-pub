#!/bin/bash

# OWL2 Reasoner Green Group Tests Runner
# This script parses green_group_tests.md and runs each test with memory monitoring

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MD_FILE="green_group_tests.md"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONITOR_SCRIPT="$SCRIPT_DIR/run_test_with_memory_monitor.sh"
MEMORY_LIMIT_MB="256"  # Memory limit set to 256MB to prevent OOM

# Counters
PASSED=0
FAILED=0
TOTAL=0
FAILED_TESTS=()

# Function to extract test name from markdown line
extract_test_name() {
    local line="$1"

    # Extract content inside backticks that ends with ': test'
    local content=$(echo "$line" | sed -n 's/.*`\([^`]*\): test`.*/\1/p')

    if [[ -z "$content" ]]; then
        return 1
    fi

    # Split by '::' and take the last part (test function name)
    local test_name=$(echo "$content" | awk -F'::' '{print $NF}')

    echo "$test_name"
}

# Validate environment
if [[ ! -f "$MD_FILE" ]]; then
    echo -e "${RED}❌ Error: $MD_FILE not found${NC}" >&2
    exit 1
fi

if [[ ! -x "$MONITOR_SCRIPT" ]]; then
    echo -e "${RED}❌ Error: $MONITOR_SCRIPT not executable or not found${NC}" >&2
    exit 1
fi

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: Please run this script from the project root directory${NC}" >&2
    exit 1
fi

echo -e "${BLUE}🧪 Running Green Group Tests${NC}"
echo -e "${BLUE}=========================${NC}"

# Process each test
while IFS= read -r line; do
    # Skip lines that don't contain test definitions
    if ! echo "$line" | grep -q ": test\`"; then
        continue
    fi

    test_name=$(extract_test_name "$line")
    if [[ -z "$test_name" ]]; then
        echo -e "${YELLOW}⚠️  Could not extract test name from: $line${NC}" >&2
        continue
    fi

    TOTAL=$((TOTAL + 1))

    echo -e "${YELLOW}🏃 Running test: $test_name${NC}"

    # Run the test with memory monitoring
    if [[ -n "$MEMORY_LIMIT_MB" ]]; then
        "$MONITOR_SCRIPT" "$test_name" "$MEMORY_LIMIT_MB"
    else
        "$MONITOR_SCRIPT" "$test_name"
    fi

    exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        echo -e "${GREEN}✅ $test_name PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}❌ $test_name FAILED (exit code: $exit_code)${NC}"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("$test_name")
        # Stop execution on first failure or memory limit exceeded
        break
    fi

done < "$MD_FILE"

# Generate summary report
echo ""
echo -e "${BLUE}📊 Summary Report${NC}"
echo -e "${BLUE}================${NC}"
echo "Total tests processed: $TOTAL"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"

if [[ ${#FAILED_TESTS[@]} -gt 0 ]]; then
    echo "Failed tests:"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "  ${RED}- $test${NC}"
    done
fi

# Exit with failure if any test failed
if [[ $FAILED -gt 0 ]]; then
    echo -e "${RED}💥 Test execution stopped due to failure${NC}"
    exit 1
else
    echo -e "${GREEN}🎉 All tests passed!${NC}"
fi