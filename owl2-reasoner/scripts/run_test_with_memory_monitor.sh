#!/bin/bash

# OWL2 Reasoner Test Runner with Memory Monitoring
# This script runs individual cargo tests with memory monitoring to detect OOM issues

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
MEMORY_LIMIT_MB=""
THRESHOLD_PERCENT=90
MONITOR_INTERVAL=2
LOG_FILE="/tmp/memory_monitor_$$.log"

# Cleanup function
cleanup() {
    # Kill monitoring process if running
    if [[ -n "$MONITOR_PID" ]] && kill -0 "$MONITOR_PID" 2>/dev/null; then
        kill "$MONITOR_PID" 2>/dev/null || true
    fi
    # Kill test process if running
    if [[ -n "$TEST_PID" ]] && kill -0 "$TEST_PID" 2>/dev/null; then
        kill "$TEST_PID" 2>/dev/null || true
    fi
    # Remove log file
    [[ -f "$LOG_FILE" ]] && rm -f "$LOG_FILE"
}

# Set trap for cleanup
trap cleanup EXIT INT TERM

# Usage function
usage() {
    cat <<EOF
Usage: $0 <test_name> [memory_limit_mb]

Arguments:
    test_name       Name of the test to run (required)
    memory_limit_mb Memory limit in MB (optional)

Options:
    -h, --help      Show this help message
    -t, --threshold Set threshold percentage for memory warning (default: 90)
    -i, --interval  Set monitoring interval in seconds (default: 2)

Examples:
    $0 my_test
    $0 my_test 512
    $0 my_test 1024 -t 80 -i 1

This script runs a cargo test with memory monitoring on macOS.
EOF
}

# Parse arguments
TEST_NAME=""
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -t|--threshold)
            THRESHOLD_PERCENT="$2"
            shift 2
            ;;
        -i|--interval)
            MONITOR_INTERVAL="$2"
            shift 2
            ;;
        -*)
            echo -e "${RED}❌ Unknown option: $1${NC}" >&2
            usage >&2
            exit 1
            ;;
        *)
            if [[ -z "$TEST_NAME" ]]; then
                TEST_NAME="$1"
            elif [[ -z "$MEMORY_LIMIT_MB" ]]; then
                MEMORY_LIMIT_MB="$1"
            else
                echo -e "${RED}❌ Too many arguments${NC}" >&2
                usage >&2
                exit 1
            fi
            shift
            ;;
    esac
done

# Validate test name
if [[ -z "$TEST_NAME" ]]; then
    echo -e "${RED}❌ Test name is required${NC}" >&2
    usage >&2
    exit 1
fi

# Validate threshold
if ! [[ "$THRESHOLD_PERCENT" =~ ^[0-9]+$ ]] || [[ "$THRESHOLD_PERCENT" -lt 1 ]] || [[ "$THRESHOLD_PERCENT" -gt 100 ]]; then
    echo -e "${RED}❌ Invalid threshold percentage: $THRESHOLD_PERCENT${NC}" >&2
    exit 1
fi

# Validate interval
if ! [[ "$MONITOR_INTERVAL" =~ ^[0-9]+$ ]] || [[ "$MONITOR_INTERVAL" -lt 1 ]]; then
    echo -e "${RED}❌ Invalid monitoring interval: $MONITOR_INTERVAL${NC}" >&2
    exit 1
fi

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: Please run this script from the project root directory${NC}" >&2
    exit 1
fi

echo -e "${BLUE}🧪 Running test '$TEST_NAME' with memory monitoring${NC}"
echo -e "${BLUE}========================================${NC}"

# Set memory limit if specified
if [[ -n "$MEMORY_LIMIT_MB" ]]; then
    if ! [[ "$MEMORY_LIMIT_MB" =~ ^[0-9]+$ ]] || [[ "$MEMORY_LIMIT_MB" -lt 1 ]]; then
        echo -e "${RED}❌ Invalid memory limit: $MEMORY_LIMIT_MB${NC}" >&2
        exit 1
    fi

    MEMORY_LIMIT_KB=$((MEMORY_LIMIT_MB * 1024))
    THRESHOLD_KB=$((MEMORY_LIMIT_KB * THRESHOLD_PERCENT / 100))

    echo -e "${YELLOW}🔒 Attempting to set memory limit to ${MEMORY_LIMIT_MB}MB (${MEMORY_LIMIT_KB}KB)${NC}"
    echo -e "${YELLOW}⚠️  Will kill test if memory exceeds ${THRESHOLD_PERCENT}% (${THRESHOLD_KB}KB)${NC}"

    # Try to set virtual memory limit, fall back to data segment limit on macOS
    if ! ulimit -v "$MEMORY_LIMIT_KB" 2>/dev/null; then
        echo -e "${YELLOW}⚠️  Virtual memory limit not supported, trying data segment limit...${NC}"
        if ! ulimit -d "$MEMORY_LIMIT_KB" 2>/dev/null; then
            echo -e "${YELLOW}⚠️  Memory limits not enforceable on this system, monitoring only${NC}"
        else
            echo -e "${GREEN}✅ Data segment limit set successfully${NC}"
        fi
    else
        echo -e "${GREEN}✅ Virtual memory limit set successfully${NC}"
    fi
else
    echo -e "${YELLOW}ℹ️  No memory limit set${NC}"
    THRESHOLD_KB=""
fi

# Initialize log file
echo "timestamp,pid,rss_kb,vsz_kb,pcpu,pmem" > "$LOG_FILE"

# Memory monitoring function
monitor_memory() {
    local test_pid=$1
    local max_rss=0
    local killed=0

    while kill -0 "$test_pid" 2>/dev/null; do
        # Get memory info using ps
        local mem_info
        mem_info=$(ps -o pid,rss,vsz,pcpu,pmem -p "$test_pid" 2>/dev/null | tail -n1)

        if [[ -n "$mem_info" ]]; then
            # Parse the output (skip header)
            local pid rss vsz pcpu pmem
            read -r pid rss vsz pcpu pmem <<< "$mem_info"

            if [[ "$pid" == "$test_pid" ]]; then
                local timestamp
                timestamp=$(date +%s)

                # Log to file
                echo "$timestamp,$pid,$rss,$vsz,$pcpu,$pmem" >> "$LOG_FILE"

                # Update max RSS
                if [[ $rss -gt $max_rss ]]; then
                    max_rss=$rss
                fi

                # Check threshold if limit is set
                if [[ -n "$THRESHOLD_KB" ]] && [[ $rss -gt $THRESHOLD_KB ]]; then
                    echo -e "${RED}🚨 Memory threshold exceeded! RSS: ${rss}KB > ${THRESHOLD_KB}KB${NC}" >&2
                    kill "$test_pid" 2>/dev/null || true
                    killed=1
                    break
                fi
            fi
        fi

        sleep "$MONITOR_INTERVAL"
    done

    # Output max memory used
    echo "$max_rss"
}

# Run the test in background
echo -e "${YELLOW}🏃 Starting test execution...${NC}"
cargo test --lib "$TEST_NAME" &
TEST_PID=$!

# Start monitoring in background
monitor_memory "$TEST_PID" &
MONITOR_PID=$!

# Wait for test to complete
TEST_OUTPUT=""
TEST_EXIT_CODE=0
if ! TEST_OUTPUT=$(wait "$TEST_PID" 2>&1); then
    TEST_EXIT_CODE=$?
fi

# Wait a moment for monitoring to finish
sleep 1

# Get max memory from monitoring
MAX_MEMORY_KB=$(tail -n1 "$LOG_FILE" | cut -d',' -f3 2>/dev/null || echo "0")

# Report results
echo ""
echo -e "${BLUE}📊 Test Results${NC}"
echo -e "${BLUE}=============${NC}"

if [[ $TEST_EXIT_CODE -eq 0 ]]; then
    echo -e "${GREEN}✅ Test PASSED${NC}"
else
    echo -e "${RED}❌ Test FAILED (exit code: $TEST_EXIT_CODE)${NC}"
fi

if [[ -n "$MAX_MEMORY_KB" ]] && [[ "$MAX_MEMORY_KB" != "0" ]]; then
    MAX_MEMORY_MB=$((MAX_MEMORY_KB / 1024))
    echo -e "${BLUE}💾 Maximum memory used: ${MAX_MEMORY_MB}MB (${MAX_MEMORY_KB}KB)${NC}"

    if [[ -n "$MEMORY_LIMIT_MB" ]]; then
        USAGE_PERCENT=$((MAX_MEMORY_KB * 100 / MEMORY_LIMIT_KB))
        echo -e "${BLUE}📈 Memory usage: ${USAGE_PERCENT}% of limit${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Could not determine memory usage${NC}"
fi

# Show memory log if available
if [[ -f "$LOG_FILE" ]] && [[ $(wc -l < "$LOG_FILE") -gt 1 ]]; then
    echo ""
    echo -e "${BLUE}📝 Memory usage log:${NC}"
    tail -n +2 "$LOG_FILE" | while IFS=',' read -r timestamp pid rss vsz pcpu pmem; do
        time_str=$(date -r "$timestamp" +%H:%M:%S 2>/dev/null || echo "$timestamp")
        rss_mb=$((rss / 1024))
        echo "  $time_str: ${rss_mb}MB RSS, ${pmem}% MEM, ${pcpu}% CPU"
    done
fi

# Exit with test's exit code
exit $TEST_EXIT_CODE