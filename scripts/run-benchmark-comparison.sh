#!/bin/bash
# Complete benchmark execution: Native ProvChain + Docker Baselines
#
# This script orchestrates the full baseline comparison benchmark:
# 1. Starts native ProvChain service
# 2. Starts Docker baseline services (Neo4j, Jena, Ethereum)
# 3. Runs the Python benchmark script
# 4. Collects and summarizes results
# 5. Cleans up all services
#
# Usage: ./scripts/run-benchmark-comparison.sh

set -e

# =============================================================================
# CONFIGURATION
# =============================================================================

SCRIPT_DIR="/home/cit/provchain-org/scripts"
PUB_DIR="/home/cit/provchain-org/docs/publication"
RESULTS_DIR="$PUB_DIR/results"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# =============================================================================
# UTILITY FUNCTIONS
# =============================================================================

log_info() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} ✓ $1"
}

log_warning() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')]${NC} ⚠ $1"
}

log_error() {
    echo -e "${RED}[$(date +'%H:%M:%S')]${NC} ✗ $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# =============================================================================
# CLEANUP FUNCTION
# =============================================================================

cleanup() {
    log_info "Cleaning up services..."

    # Stop ProvChain
    if [ -f "$SCRIPT_DIR/provchain-service.sh" ]; then
        $SCRIPT_DIR/provchain-service.sh stop
    fi

    # Stop Docker baselines
    cd "$PUB_DIR"
    docker-compose -f docker-compose.baselines-only.yml down

    log_success "Cleanup complete"
}

# Set trap to cleanup on exit
trap cleanup EXIT INT TERM

# =============================================================================
# MAIN EXECUTION
# =============================================================================

print_header "${BOLD}Baseline Comparison Benchmark Runner${NC}"
echo -e "Architecture: ${GREEN}Native ProvChain${NC} + ${YELLOW}Docker Baselines${NC}"
echo ""

# =============================================================================
# PHASE 1: START NATIVE PROVCHAIN
# =============================================================================

print_header "Phase 1: Starting Native ProvChain"

if [ ! -f "$SCRIPT_DIR/provchain-service.sh" ]; then
    log_error "provchain-service.sh not found at $SCRIPT_DIR/provchain-service.sh"
    exit 1
fi

# Start ProvChain service
log_info "Starting native ProvChain service..."
$SCRIPT_DIR/provchain-service.sh start

# Verify ProvChain is running
log_info "Verifying ProvChain health..."
sleep 3

if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    log_success "ProvChain is running and responding"
else
    log_error "ProvChain failed to start"
    log_info "Check logs: tail -f /tmp/provchain.log"
    exit 1
fi

# Run quick test
log_info "Running quick functionality test..."
if [ -f "$SCRIPT_DIR/test-provchain.sh" ]; then
    $SCRIPT_DIR/test-provchain.sh > /tmp/provchain-test.log 2>&1
    log_success "ProvChain test complete"
fi

echo ""

# =============================================================================
# PHASE 2: START DOCKER BASELINES
# =============================================================================

print_header "Phase 2: Starting Docker Baselines"

if [ ! -f "$PUB_DIR/docker-compose.baselines-only.yml" ]; then
    log_error "docker-compose.baselines-only.yml not found"
    exit 1
fi

cd "$PUB_DIR"

log_info "Starting Docker baseline services..."
docker-compose -f docker-compose.baselines-only.yml up -d

# Wait for services to be ready
log_info "Waiting for baseline services to be ready..."
sleep 15

# Verify Neo4j
log_info "Checking Neo4j..."
if curl -s http://localhost:7474 > /dev/null 2>&1; then
    log_success "Neo4j is running"
else
    log_warning "Neo4j not responding - may still be starting"
fi

# Verify Jena
log_info "Checking Jena Fuseki..."
if curl -s http://localhost:3030 > /dev/null 2>&1; then
    log_success "Jena Fuseki is running"
else
    log_warning "Jena Fuseki not responding - may still be starting"
fi

# Verify Ethereum
log_info "Checking Ethereum Ganache..."
if curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    > /dev/null 2>&1; then
    log_success "Ethereum Ganache is running"
else
    log_warning "Ethereum Ganache not responding - may still be starting"
fi

echo ""

# =============================================================================
# PHASE 3: RUN BENCHMARKS
# =============================================================================

print_header "Phase 3: Running Benchmarks"

log_info "Starting benchmark execution..."
log_info "This may take 15-30 minutes depending on dataset sizes"

# Create results directory if it doesn't exist
mkdir -p "$RESULTS_DIR"

# Run the Python benchmark script
python3 "$PUB_DIR/scripts/run_baseline_benchmarks.py"

# Check if benchmark completed successfully
if [ $? -eq 0 ]; then
    log_success "Benchmarks completed successfully"
else
    log_error "Benchmarks failed with exit code $?"
    log_info "Check logs above for details"
fi

echo ""

# =============================================================================
# PHASE 4: COLLECT RESULTS
# =============================================================================

print_header "Phase 4: Collecting Results"

if [ -f "$RESULTS_DIR/baseline_comparison.json" ]; then
    log_info "Results saved to: $RESULTS_DIR/baseline_comparison.json"

    # Show summary
    echo ""
    echo "Benchmark Results Summary:"
    echo "======================="
    python3 -c "
import json
with open('$RESULTS_DIR/baseline_comparison.json', 'r') as f:
    data = json.load(f)

for system in ['neo4j', 'jena', 'ethereum', 'provchain']:
    if system in data:
        print(f'\n{system.upper()}:')
        for query in ['simple_select', 'type_query', 'join_query', 'complex_join']:
            if query in data[system]:
                stats = data[system][query]
                if stats and 'mean' in stats:
                    print(f'  {query}: {stats[\"mean\"]:.2f} μs (n={stats[\"count\"]})')
" 2>/dev/null || echo "Python not available for summary"

    # Generate markdown table if available
    if [ -f "$RESULTS_DIR/COMPARISON_TABLE.md" ]; then
        echo ""
        echo "Comparison table: $RESULTS_DIR/COMPARISON_TABLE.md"
    fi

else
    log_warning "No results file found at $RESULTS_DIR/baseline_comparison.json"
fi

echo ""

# =============================================================================
# PHASE 5: SUMMARY
# =============================================================================

print_header "${BOLD}Benchmark Complete${NC}"

log_success "All benchmarks have been executed"
echo ""
echo "Results Location:"
echo "  - JSON: $RESULTS_DIR/baseline_comparison.json"
echo "  - Markdown: $RESULTS_DIR/COMPARISON_TABLE.md"
echo ""
echo "Next Steps:"
echo "  1. Review results: cat $RESULTS_DIR/COMPARISON_TABLE.md"
echo "  2. Analyze data: python3 -c \"import json; print(json.load(open('$RESULTS_DIR/baseline_comparison.json')).keys())\""
echo "  3. Update thesis with baseline comparison data"
echo ""
echo "Services Status:"
echo "  - ProvChain: Still running (use: $SCRIPT_DIR/provchain-service.sh stop)"
echo "  - Baselines: Still running (use: cd $PUB_DIR && docker-compose -f docker-compose.baselines-only.yml down)"
echo ""
echo "To stop all services:"
echo "  $SCRIPT_DIR/provchain-service.sh stop"
echo "  cd $PUB_DIR && docker-compose -f docker-compose.baselines-only.yml down"
echo ""

# =============================================================================
# END
# =============================================================================
