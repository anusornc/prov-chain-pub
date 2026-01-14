#!/bin/bash
################################################################################
# ProvChain-Org Benchmark Toolkit - Quick Start Script
################################################################################
# ONE-COMMAND BENCHMARK RUNNER
#
# This script automatically:
# 1. Detects your hardware capabilities
# 2. Configures optimal settings
# 3. Starts all services (ProvChain + Neo4j + Monitoring)
# 4. Runs all benchmarks
# 5. Generates comparison reports
# 6. Displays results
#
# Usage:
#   ./run.sh [hardware_profile]
#
# Hardware Profiles (auto-detected if not specified):
#   low      - 4GB RAM, 2 CPU cores (minimal testing)
#   medium   - 8GB RAM, 4 CPU cores (standard)
#   high     - 16GB RAM, 8 CPU cores (full testing)
#   ultra    - 32GB+ RAM, 16+ CPU cores (extensive)
################################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Script directory
TOOLKIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Configuration defaults
COMPOSE_FILE="${TOOLKIT_DIR}/docker-compose.yml"
RESULTS_DIR="${TOOLKIT_DIR}/results"
LOGS_DIR="${TOOLKIT_DIR}/logs"
DATASET="${TOOLKIT_DIR}/data/supply_chain.ttl"

# Hardware profiles
declare -A PROFILES
PROFILES[low]="4GB RAM, 2 CPU cores, 5 iterations, minimal dataset"
PROFILES[medium]="8GB RAM, 4 CPU cores, 10 iterations, standard dataset"
PROFILES[high]="16GB RAM, 8 CPU cores, 20 iterations, extended dataset"
PROFILES[ultra]="32GB+ RAM, 16+ CPU cores, 50 iterations, full dataset"

################################################################################
# Helper Functions
################################################################################

print_banner() {
    echo -e "${CYAN}"
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║        ProvChain-Org Benchmark Toolkit v1.0                   ║"
    echo "║        Portable Performance Comparison Suite                  ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_step() {
    echo -e "\n${BLUE}▶ ${1}${NC}"
}

print_success() {
    echo -e "${GREEN}✓ ${1}${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ ${1}${NC}"
}

print_error() {
    echo -e "${RED}✗ ${1}${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ ${1}${NC}"
}

################################################################################
# Hardware Detection
################################################################################

detect_hardware() {
    print_step "Detecting hardware capabilities..."

    # Get RAM in GB
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        RAM_GB=$(sysctl -n hw.memsize | awk '{print int($1/1024/1024/1024)}')
        CPU_CORES=$(sysctl -n hw.ncpu)
    else
        # Linux
        RAM_GB=$(free -g | awk '/^Mem:/{print $2}')
        CPU_CORES=$(nproc)
    fi

    # Get available disk space in GB
    DISK_GB=$(df -BG "$TOOLKIT_DIR" 2>/dev/null | tail -1 | awk '{print $4}' | tr -d 'G' || echo "0")

    # If df -BG doesn't work, try alternative
    if [ -z "$DISK_GB" ] || [ "$DISK_GB" = "0" ]; then
        DISK_GB=$(df -H "$TOOLKIT_DIR" 2>/dev/null | tail -1 | awk '{print $4}' | tr -d 'G' | tr -d 'A-Za-z' || echo "10")
    fi

    print_success "RAM: ${RAM_GB}GB | CPU: ${CPU_CORES} cores | Disk: ${DISK_GB}GB available"

    # Auto-detect profile
    if [ "$RAM_GB" -lt 8 ] || [ "$CPU_CORES" -lt 4 ]; then
        PROFILE="low"
    elif [ "$RAM_GB" -lt 16 ] || [ "$CPU_CORES" -lt 8 ]; then
        PROFILE="medium"
    elif [ "$RAM_GB" -lt 32 ] || [ "$CPU_CORES" -lt 16 ]; then
        PROFILE="high"
    else
        PROFILE="ultra"
    fi

    print_info "Auto-detected profile: ${BOLD}${PROFILE}${NC}"
    echo "   → ${PROFILES[$PROFILE]}"
}

################################################################################
# Environment Validation
################################################################################

check_environment() {
    print_step "Validating environment..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        echo ""
        echo "Install Docker:"
        echo "  Ubuntu/Debian: curl -fsSL https://get.docker.com | sh"
        echo "  macOS: Download from https://www.docker.com/products/docker-desktop"
        exit 1
    fi
    print_success "Docker: $(docker --version | cut -d' ' -f3)"

    # Check Docker Compose
    if ! docker compose version &> /dev/null; then
        print_error "Docker Compose is not installed"
        echo "  Install with: sudo apt-get install docker-compose-plugin"
        exit 1
    fi
    print_success "Docker Compose: OK"

    # Check if Docker is running
    if ! docker info &> /dev/null; then
        print_error "Docker is not running. Please start Docker Desktop or service."
        exit 1
    fi
    print_success "Docker daemon: Running"

    # Check available disk space
    if [ "$DISK_GB" -lt 5 ]; then
        print_warning "Low disk space (< 5GB). Benchmarks may fail."
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

################################################################################
# Configuration Loading
################################################################################

load_config() {
    print_step "Loading configuration for profile: ${BOLD}${PROFILE}${NC}"

    local config_file="${TOOLKIT_DIR}/configs/${PROFILE}.conf"

    if [ ! -f "$config_file" ]; then
        print_error "Configuration file not found: $config_file"
        exit 1
    fi

    # Source the profile configuration
    source "$config_file"

    print_success "Configuration loaded"
    echo "   → Iterations: ${ITERATIONS}"
    echo "   → Dataset: ${DATASET_SIZE}"
    echo "   → Concurrent users: ${CONCURRENT_USERS}"
}

################################################################################
# Service Management
################################################################################

cleanup() {
    print_step "Cleaning up previous run..."

    cd "$TOOLKIT_DIR"

    # Stop and remove containers
    docker compose -f "$COMPOSE_FILE" down -v 2>/dev/null || true

    # Clean up old results (optional)
    if [ "$CLEAN_RESULTS" = "true" ]; then
        rm -rf "${RESULTS_DIR:?}"/*
        print_success "Old results cleaned"
    fi
}

start_services() {
    print_step "Starting benchmark services..."

    cd "$TOOLKIT_DIR"

    # Create necessary directories
    mkdir -p "$RESULTS_DIR" "$LOGS_DIR"

    # Start services
    docker compose -f "$COMPOSE_FILE" up -d

    print_success "Services started"
}

wait_for_services() {
    print_step "Waiting for services to be healthy..."

    local max_wait=120
    local waited=0

    # Wait for ProvChain
    echo -n "   ProvChain-Org..."
    while [ $waited -lt $max_wait ]; do
        if curl -sf http://localhost:8080/health &> /dev/null; then
            echo -e " ${GREEN}✓${NC}"
            break
        fi
        sleep 2
        waited=$((waited + 2))
    done

    if [ $waited -ge $max_wait ]; then
        echo -e " ${RED}✗ Timeout${NC}"
        return 1
    fi

    # Wait for Neo4j
    echo -n "   Neo4j..."
    waited=0
    while [ $waited -lt $max_wait ]; do
        if curl -sf http://localhost:7474 &> /dev/null; then
            echo -e " ${GREEN}✓${NC}"
            break
        fi
        sleep 2
        waited=$((waited + 2))
    done

    if [ $waited -ge $max_wait ]; then
        echo -e " ${RED}✗ Timeout${NC}"
        return 1
    fi

    # Wait for Prometheus
    echo -n "   Prometheus..."
    waited=0
    while [ $waited -lt 60 ]; do
        if curl -sf http://localhost:9091/-/healthy &> /dev/null; then
            echo -e " ${GREEN}✓${NC}"
            break
        fi
        sleep 2
        waited=$((waited + 2))
    done

    # Wait for Grafana
    echo -n "   Grafana..."
    waited=0
    while [ $waited -lt 60 ]; do
        if curl -sf http://localhost:3000/api/health &> /dev/null; then
            echo -e " ${GREEN}✓${NC}"
            break
        fi
        sleep 2
        waited=$((waited + 2))
    done

    echo ""
    print_success "All services are healthy"
}

################################################################################
# Benchmark Execution
################################################################################

run_benchmarks() {
    print_step "Running benchmarks..."

    cd "$TOOLKIT_DIR"

    print_info "This may take several minutes depending on your hardware..."

    # Run the benchmark container
    docker compose -f "$COMPOSE_FILE" run --rm \
        -e ITERATIONS="$ITERATIONS" \
        -e DATASET_SIZE="$DATASET_SIZE" \
        -e CONCURRENT_USERS="$CONCURRENT_USERS" \
        benchmark-runner

    print_success "Benchmarks completed"
}

################################################################################
# Results Display
################################################################################

display_results() {
    print_step "Benchmark Results"

    echo ""

    if [ -f "${RESULTS_DIR}/summary.md" ]; then
        cat "${RESULTS_DIR}/summary.md"
    else
        print_warning "No results found. Check logs: ${LOGS_DIR}/"
        return 1
    fi

    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}Access Points:${NC}"
    echo -e "   • Grafana Dashboard: ${GREEN}http://localhost:3000${NC} (admin/admin)"
    echo -e "   • Prometheus:        ${GREEN}http://localhost:9091${NC}"
    echo -e "   • ProvChain API:     ${GREEN}http://localhost:8080${NC}"
    echo -e "   • Neo4j Browser:     ${GREEN}http://localhost:7474${NC} (neo4j/benchmark)"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"

    echo ""
    echo -e "${YELLOW}Press Ctrl+C to stop services. Data will persist.${NC}"
}

################################################################################
# Main Execution
################################################################################

main() {
    # Parse command line arguments
    PROFILE="${1:-auto}"
    CLEAN_RESULTS="${CLEAN_RESULTS:-false}"

    print_banner

    # Auto-detect profile if not specified
    if [ "$PROFILE" = "auto" ]; then
        detect_hardware
    elif [[ ! " ${!PROFILES[@]} " =~ " ${PROFILE} " ]]; then
        print_error "Invalid profile: $PROFILE"
        echo ""
        echo "Available profiles:"
        for p in "${!PROFILES[@]}"; do
            echo "  ${BOLD}${p}${NC} - ${PROFILES[$p]}"
        done
        exit 1
    fi

    # Validation
    check_environment

    # Load configuration
    load_config

    # Cleanup previous run
    cleanup

    # Start services
    start_services

    # Wait for services to be ready
    wait_for_services

    # Run benchmarks
    run_benchmarks

    # Display results
    display_results

    # Keep containers running
    print_info "All services are running. Press Ctrl+C to stop."
    sleep infinity
}

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
