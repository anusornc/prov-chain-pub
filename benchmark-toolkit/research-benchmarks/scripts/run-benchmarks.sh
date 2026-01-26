#!/bin/bash
# Benchmark Runner Script
# Executes all benchmarks and generates comparison reports

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="${PROJECT_ROOT}/../deploy/docker-compose.benchmark-comparison.yml"
RESULTS_DIR="${PROJECT_ROOT}/results"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     ProvChain-Org Benchmark Runner${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Parse arguments
BENCHMARK_TYPE="${1:-all}"
ITERATIONS="${2:-10}"

echo -e "${GREEN}Configuration:${NC}"
echo "  Benchmark Type: ${BENCHMARK_TYPE}"
echo "  Iterations: ${ITERATIONS}"
echo "  Compose File: ${COMPOSE_FILE}"
echo "  Results Dir: ${RESULTS_DIR}"
echo ""

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        echo -e "${RED}Error: Docker is not running${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Docker is running${NC}"
}

# Function to cleanup containers
cleanup() {
    echo -e "${YELLOW}Cleaning up containers...${NC}"
    cd "${PROJECT_ROOT}/../deploy"
    docker-compose -f docker-compose.benchmark-comparison.yml down -v
}

# Function to wait for services to be healthy
wait_for_services() {
    echo -e "${YELLOW}Waiting for services to be healthy...${NC}"

    # Wait for ProvChain
    echo -n "  ProvChain-Org..."
    for i in {1..60}; do
        if curl -sf http://localhost:8080/health > /dev/null 2>&1; then
            echo -e " ${GREEN}✓${NC}"
            break
        fi
        if [ $i -eq 60 ]; then
            echo -e " ${RED}✗ Timed out${NC}"
            exit 1
        fi
        sleep 2
    done

    # Wait for Neo4j
    echo -n "  Neo4j..."
    for i in {1..60}; do
        if curl -sf http://localhost:7474 > /dev/null 2>&1; then
            echo -e " ${GREEN}✓${NC}"
            break
        fi
        if [ $i -eq 60 ]; then
            echo -e " ${RED}✗ Timed out${NC}"
            exit 1
        fi
        sleep 2
    done

    # Wait for Prometheus
    echo -n "  Prometheus..."
    for i in {1..30}; do
        if curl -sf http://localhost:9092/-/healthy > /dev/null 2>&1; then
            echo -e " ${GREEN}✓${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e " ${RED}✗ Timed out${NC}"
            exit 1
        fi
        sleep 2
    done

    # Wait for Grafana
    echo -n "  Grafana..."
    for i in {1..30}; do
        if curl -sf http://localhost:3002/api/health > /dev/null 2>&1; then
            echo -e " ${GREEN}✓${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e " ${RED}✗ Timed out${NC}"
            exit 1
        fi
        sleep 2
    done

    echo ""
}

# Function to run benchmarks
run_benchmarks() {
    echo -e "${BLUE}Starting benchmark execution...${NC}"
    echo ""

    cd "${PROJECT_ROOT}/../deploy"

    # Run the benchmark runner container
    docker-compose -f docker-compose.benchmark-comparison.yml \
        run --rm \
        -e ITERATIONS="${ITERATIONS}" \
        benchmark-runner \
        --${BENCHMARK_TYPE}

    echo ""
    echo -e "${GREEN}✓ Benchmarks completed${NC}"
}

# Function to display results
display_results() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}     Benchmark Results${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
    echo ""

    # Display summary if it exists
    if [ -f "${RESULTS_DIR}/summary.md" ]; then
        cat "${RESULTS_DIR}/summary.md"
    else
        echo -e "${YELLOW}No results found${NC}"
    fi

    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}Results Location: ${RESULTS_DIR}${NC}"
    echo -e "${GREEN}Grafana Dashboard: http://localhost:3002/d/provchain-benchmark${NC}"
    echo -e "${GREEN}Prometheus: http://localhost:9092${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
}

# Main execution
main() {
    check_docker

    # Trap to cleanup on exit
    trap cleanup EXIT

    echo -e "${YELLOW}Step 1: Starting services...${NC}"
    cd "${PROJECT_ROOT}/../deploy"
    docker-compose -f docker-compose.benchmark-comparison.yml up -d

    echo ""
    wait_for_services

    echo -e "${YELLOW}Step 2: Running benchmarks...${NC}"
    run_benchmarks

    echo -e "${YELLOW}Step 3: Displaying results...${NC}"
    display_results

    echo ""
    echo -e "${GREEN}Benchmark run complete!${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop services and cleanup${NC}"

    # Keep containers running for inspection
    sleep infinity
}

# Run main function
main "$@"
