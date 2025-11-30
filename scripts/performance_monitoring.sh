#!/bin/bash

# Performance Monitoring and Metrics Collection Script for ProvChain-Org
#
# This script provides comprehensive performance monitoring including:
# - System resource monitoring
# - Application performance metrics
# - Database query performance
# - Network performance monitoring
# - Real-time alerting

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$PROJECT_ROOT/logs/performance"
METRICS_DIR="$PROJECT_ROOT/metrics"
ALERT_THRESHOLD_CPU=80
ALERT_THRESHOLD_MEMORY=85
ALERT_THRESHOLD_DISK=90
MONITORING_INTERVAL=5  # seconds

# Create directories
mkdir -p "$LOG_DIR"
mkdir -p "$METRICS_DIR"

# Log file with timestamp
LOG_FILE="$LOG_DIR/performance_$(date +%Y%m%d_%H%M%S).log"
METRICS_FILE="$METRICS_DIR/metrics_$(date +%Y%m%d_%H%M%S).json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    case $level in
        "INFO")
            echo -e "${GREEN}[INFO]${NC} $timestamp - $message" | tee -a "$LOG_FILE"
            ;;
        "WARN")
            echo -e "${YELLOW}[WARN]${NC} $timestamp - $message" | tee -a "$LOG_FILE"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} $timestamp - $message" | tee -a "$LOG_FILE"
            ;;
        "DEBUG")
            echo -e "${BLUE}[DEBUG]${NC} $timestamp - $message" | tee -a "$LOG_FILE"
            ;;
    esac
}

# System resource monitoring
monitor_system_resources() {
    log "INFO" "Starting system resource monitoring"

    while true; do
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

        # CPU usage
        local cpu_usage=$(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}')

        # Memory usage
        local memory_info=$(free -m | awk 'NR==2{printf "%.2f", $3*100/$2}')
        local memory_used=$(free -m | awk 'NR==2{print $3}')
        local memory_total=$(free -m | awk 'NR==2{print $2}')

        # Disk usage
        local disk_usage=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')

        # Load average
        local load_avg=$(uptime | awk -F'load average:' '{print $2}' | awk '{print $1}' | sed 's/,//')

        # Network I/O
        local network_rx=$(cat /proc/net/dev | grep eth0 | awk '{print $2}' || echo "0")
        local network_tx=$(cat /proc/net/dev | grep eth0 | awk '{print $10}' || echo "0")

        # Create metrics JSON
        local metrics=$(cat <<EOF
{
  "timestamp": "$timestamp",
  "system": {
    "cpu_usage_percent": $cpu_usage,
    "memory_usage_percent": $memory_usage,
    "memory_used_mb": $memory_used,
    "memory_total_mb": $memory_total,
    "disk_usage_percent": $disk_usage,
    "load_average": $load_avg,
    "network_rx_bytes": $network_rx,
    "network_tx_bytes": $network_tx
  }
}
EOF
)

        echo "$metrics" >> "$METRICS_FILE"

        # Check for alerts
        check_alerts "$cpu_usage" "$memory_usage" "$disk_usage"

        # Log current metrics
        log "INFO" "CPU: ${cpu_usage}%, Memory: ${memory_usage}%, Disk: ${disk_usage}%, Load: ${load_avg}"

        sleep "$MONITORING_INTERVAL"
    done
}

# Alert checking function
check_alerts() {
    local cpu=$1
    local memory=$2
    local disk=$3

    if (( $(echo "$cpu > $ALERT_THRESHOLD_CPU" | bc -l) )); then
        log "WARN" "High CPU usage detected: ${cpu}%"
        send_alert "High CPU Usage" "CPU usage is ${cpu}% (threshold: ${ALERT_THRESHOLD_CPU}%)"
    fi

    if (( $(echo "$memory > $ALERT_THRESHOLD_MEMORY" | bc -l) )); then
        log "WARN" "High memory usage detected: ${memory}%"
        send_alert "High Memory Usage" "Memory usage is ${memory}% (threshold: ${ALERT_THRESHOLD_MEMORY}%)"
    fi

    if (( disk > ALERT_THRESHOLD_DISK )); then
        log "WARN" "High disk usage detected: ${disk}%"
        send_alert "High Disk Usage" "Disk usage is ${disk}% (threshold: ${ALERT_THRESHOLD_DISK}%)"
    fi
}

# Alert sending function
send_alert() {
    local title=$1
    local message=$2

    # Log alert
    log "WARN" "ALERT: $title - $message"

    # Here you can integrate with your alerting system
    # Examples: Slack, Email, PagerDuty, etc.

    # Example: Send to Slack (requires webhook URL)
    if [[ -n "${SLACK_WEBHOOK_URL:-}" ]]; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"🚨 $title: $message\"}" \
            "$SLACK_WEBHOOK_URL" 2>/dev/null || true
    fi

    # Example: Send email (requires mail configuration)
    if command -v mail &> /dev/null && [[ -n "${ALERT_EMAIL:-}" ]]; then
        echo "$message" | mail -s "ProvChain Alert: $title" "$ALERT_EMAIL" 2>/dev/null || true
    fi
}

# Application-specific monitoring
monitor_application() {
    log "INFO" "Starting application-specific monitoring"

    local provchain_pid=$(pgrep -f "provchain-org" || echo "")

    if [[ -z "$provchain_pid" ]]; then
        log "WARN" "ProvChain application not running"
        return 1
    fi

    log "INFO" "Monitoring ProvChain process (PID: $provchain_pid)"

    while true; do
        if ! kill -0 "$provchain_pid" 2>/dev/null; then
            log "ERROR" "ProvChain process has stopped"
            send_alert "Application Down" "ProvChain application is no longer running"
            break
        fi

        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

        # Process-specific metrics
        local process_cpu=$(ps -p "$provchain_pid" -o %cpu --no-headers | tr -d ' ')
        local process_memory=$(ps -p "$provchain_pid" -o %mem --no-headers | tr -d ' ')
        local process_threads=$(ps -p "$provchain_pid" -o nlwp --no-headers | tr -d ' ')
        local process_fd=$(lsof -p "$provchain_pid" 2>/dev/null | wc -l)

        # Application-specific metrics
        local app_metrics=$(curl -s "http://localhost:8080/api/metrics" 2>/dev/null || echo "{}")

        local full_metrics=$(cat <<EOF
{
  "timestamp": "$timestamp",
  "process": {
    "pid": $provchain_pid,
    "cpu_usage_percent": $process_cpu,
    "memory_usage_percent": $process_memory,
    "thread_count": $process_threads,
    "file_descriptor_count": $process_fd
  },
  "application": $app_metrics
}
EOF
)

        echo "$full_metrics" >> "$METRICS_FILE"

        log "INFO" "Process CPU: ${process_cpu}%, Memory: ${process_memory}%, Threads: ${process_threads}, FDs: ${process_fd}"

        sleep "$MONITORING_INTERVAL"
    done
}

# Database performance monitoring
monitor_database() {
    log "INFO" "Starting database performance monitoring"

    while true; do
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

        # Check if database is accessible
        if ! curl -s "http://localhost:7878/repositories" >/dev/null 2>&1; then
            log "WARN" "Database not accessible"
            sleep 10
            continue
        fi

        # Get database statistics
        local db_stats=$(curl -s "http://localhost:7878/repositories/provchain/stats" 2>/dev/null || echo "{}")

        # Query performance test
        local query_start=$(date +%s%N)
        local query_result=$(curl -s -X POST "http://localhost:7878/repositories/provchain" \
            -H "Content-Type: application/sparql-query" \
            -H "Accept: application/sparql-results+json" \
            -d "SELECT (COUNT(*) as ?triples) WHERE { ?s ?p ?o }" 2>/dev/null)
        local query_end=$(date +%s%N)
        local query_time=$(( (query_end - query_start) / 1000000 )) # Convert to milliseconds

        local db_metrics=$(cat <<EOF
{
  "timestamp": "$timestamp",
  "database": {
    "query_time_ms": $query_time,
    "stats": $db_stats
  }
}
EOF
)

        echo "$db_metrics" >> "$METRICS_FILE"

        if (( query_time > 5000 )); then
            log "WARN" "Slow database query detected: ${query_time}ms"
        fi

        sleep 30 # Database checks less frequently
    done
}

# Network performance monitoring
monitor_network() {
    log "INFO" "Starting network performance monitoring"

    while true; do
        local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

        # HTTP response time
        local http_start=$(date +%s%N)
        local http_status=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:8080/api/health" || echo "000")
        local http_end=$(date +%s%N)
        local http_time=$(( (http_end - http_start) / 1000000 ))

        # WebSocket connection test
        local ws_start=$(date +%s%N)
        local ws_result=$(timeout 5s curl -s -I "http://localhost:8080/ws" 2>/dev/null || echo "failed")
        local ws_end=$(date +%s%N)
        local ws_time=$(( (ws_end - ws_start) / 1000000 ))

        local network_metrics=$(cat <<EOF
{
  "timestamp": "$timestamp",
  "network": {
    "http_response_time_ms": $http_time,
    "http_status_code": $http_status,
    "websocket_response_time_ms": $ws_time,
    "websocket_status": "$ws_result"
  }
}
EOF
)

        echo "$network_metrics" >> "$METRICS_FILE"

        if [[ "$http_status" != "200" ]]; then
            log "WARN" "HTTP health check failed with status: $http_status"
        fi

        if (( http_time > 1000 )); then
            log "WARN" "Slow HTTP response time: ${http_time}ms"
        fi

        sleep 10
    done
}

# Performance report generation
generate_performance_report() {
    log "INFO" "Generating performance report"

    local report_file="$METRICS_DIR/performance_report_$(date +%Y%m%d_%H%M%S).html"

    cat > "$report_file" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>ProvChain Performance Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .chart-container { width: 800px; height: 400px; margin: 20px 0; }
        .metric { margin: 10px 0; padding: 10px; border: 1px solid #ddd; }
        .alert { color: #d32f2f; }
        .warning { color: #f57c00; }
        .info { color: #1976d2; }
    </style>
</head>
<body>
    <h1>ProvChain Performance Report</h1>
    <div id="report-content">
        <p>Generating report...</p>
    </div>

    <div class="chart-container">
        <canvas id="cpuChart"></canvas>
    </div>
    <div class="chart-container">
        <canvas id="memoryChart"></canvas>
    </div>
    <div class="chart-container">
        <canvas id="responseTimeChart"></canvas>
    </div>

    <script>
        // This would be populated with actual metrics data
        // For now, showing the structure
        const cpuCtx = document.getElementById('cpuChart').getContext('2d');
        new Chart(cpuCtx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'CPU Usage %',
                    data: [],
                    borderColor: 'rgb(75, 192, 192)',
                    tension: 0.1
                }]
            },
            options: {
                responsive: true,
                plugins: {
                    title: {
                        display: true,
                        text: 'CPU Usage Over Time'
                    }
                }
            }
        });

        // Similar charts for memory and response times...
    </script>
</body>
</html>
EOF

    log "INFO" "Performance report generated: $report_file"
}

# Cleanup function
cleanup() {
    log "INFO" "Cleaning up monitoring processes"

    # Kill all background processes
    jobs -p | xargs -r kill

    log "INFO" "Performance monitoring stopped"
    exit 0
}

# Signal handlers
trap cleanup SIGINT SIGTERM

# Main execution
main() {
    log "INFO" "Starting ProvChain performance monitoring"
    log "INFO" "Log file: $LOG_FILE"
    log "INFO" "Metrics file: $METRICS_FILE"

    # Check dependencies
    local missing_deps=()

    for cmd in bc curl top free df; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log "ERROR" "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi

    # Start monitoring in background
    monitor_system_resources &
    SYSTEM_MONITOR_PID=$!

    sleep 2
    monitor_application &
    APP_MONITOR_PID=$!

    sleep 2
    monitor_database &
    DB_MONITOR_PID=$!

    sleep 2
    monitor_network &
    NETWORK_MONITOR_PID=$!

    log "INFO" "All monitoring processes started"
    log "INFO" "System monitor PID: $SYSTEM_MONITOR_PID"
    log "INFO" "App monitor PID: $APP_MONITOR_PID"
    log "INFO" "DB monitor PID: $DB_MONITOR_PID"
    log "INFO" "Network monitor PID: $NETWORK_MONITOR_PID"

    # Generate periodic reports
    while true; do
        sleep 300  # 5 minutes
        generate_performance_report
    done
}

# Usage function
usage() {
    cat << EOF
ProvChain Performance Monitoring Script

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -i, --interval SECONDS  Set monitoring interval (default: 5)
    -c, --cpu-threshold     CPU alert threshold percentage (default: 80)
    -m, --memory-threshold  Memory alert threshold percentage (default: 85)
    -d, --disk-threshold    Disk alert threshold percentage (default: 90)
    --report-only           Generate report only (no monitoring)

ENVIRONMENT VARIABLES:
    SLACK_WEBHOOK_URL       Slack webhook URL for alerts
    ALERT_EMAIL             Email address for alerts

EXAMPLES:
    $0                                    # Start with default settings
    $0 --interval 10                      # Check every 10 seconds
    $0 --cpu-threshold 90                 # Alert at 90% CPU
    $0 --report-only                     # Generate report only

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -i|--interval)
            MONITORING_INTERVAL="$2"
            shift 2
            ;;
        -c|--cpu-threshold)
            ALERT_THRESHOLD_CPU="$2"
            shift 2
            ;;
        -m|--memory-threshold)
            ALERT_THRESHOLD_MEMORY="$2"
            shift 2
            ;;
        -d|--disk-threshold)
            ALERT_THRESHOLD_DISK="$2"
            shift 2
            ;;
        --report-only)
            generate_performance_report
            exit 0
            ;;
        *)
            log "ERROR" "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Run main function
main "$@"