#!/bin/bash
# Benchmark Results Analyzer
# Post-processes benchmark results and generates detailed reports

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="${PROJECT_ROOT}/results"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     Benchmark Results Analyzer${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Check if results exist
if [ ! -f "${RESULTS_DIR}/benchmark_results.json" ]; then
    echo -e "${RED}Error: No benchmark results found${NC}"
    echo "Run './scripts/run-benchmarks.sh' first"
    exit 1
fi

# Parse JSON and generate statistics
echo -e "${GREEN}Analyzing benchmark results...${NC}"
echo ""

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}Installing jq for JSON processing...${NC}"
    sudo apt-get update && sudo apt-get install -y jq
fi

# Extract metrics
echo -e "${BLUE}Metrics Summary:${NC}"
echo ""

# Total operations
TOTAL_OPS=$(jq '[.[] | .operations_per_second] | add' "${RESULTS_DIR}/benchmark_results.json")
echo "  Total Operations/sec: ${TOTAL_OPS}"

# Average duration
AVG_DURATION=$(jq '[.[] | .duration_ms] | add / length' "${RESULTS_DIR}/benchmark_results.json")
echo "  Average Duration: ${AVG_DURATION} ms"

# Success rate
SUCCESS_COUNT=$(jq '[.[] | select(.success == true)] | length' "${RESULTS_DIR}/benchmark_results.json")
TOTAL_COUNT=$(jq '. | length' "${RESULTS_DIR}/benchmark_results.json")
SUCCESS_RATE=$(echo "scale=2; $SUCCESS_COUNT * 100 / $TOTAL_COUNT" | bc)
echo "  Success Rate: ${SUCCESS_RATE}%"

echo ""
echo -e "${BLUE}Performance by Scenario:${NC}"
echo ""

# Group by scenario
jq -r '
  group_by(.scenario) |
  .[] |
  {
    scenario: .[0].scenario,
    avg_duration: ([.[].duration_ms] | add / length | floor),
    avg_ops: ([.[].operations_per_second] | add / length | floor),
    min_duration: ([.[].duration_ms] | min | floor),
    max_duration: ([.[].duration_ms] | max | floor)
  } |
  "  \(.scenario):\n    Average: \(.avg_duration) ms (\(.avg_ops) ops/sec)\n    Min: \(.min_duration) ms\n    Max: \(.max_duration) ms\n"
' "${RESULTS_DIR}/benchmark_results.json"

echo ""
echo -e "${GREEN}✓ Analysis complete${NC}"
echo ""
echo -e "${BLUE}Detailed results available:${NC}"
echo "  - ${RESULTS_DIR}/benchmark_results.json"
echo "  - ${RESULTS_DIR}/benchmark_results.csv"
echo "  - ${RESULTS_DIR}/summary.md"
echo ""
