#!/bin/bash
# Service management for NATIVE ProvChain
# This script manages ProvChain running natively (not in Docker)

set -e

PROVCHAIN_DIR="/home/cit/provchain-org"
PROVCHAIN_PORT=8080
PROVCHAIN_PID_FILE="/tmp/provchain.pid"
PROVCHAIN_LOG_FILE="/tmp/provchain.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

start() {
    echo -e "${YELLOW}Starting native ProvChain...${NC}"

    # Check if we're in the correct directory
    if [ ! -f "$PROVCHAIN_DIR/Cargo.toml" ]; then
        echo -e "${RED}Error: Not in ProvChain directory${NC}"
        echo "Expected: $PROVCHAIN_DIR"
        return 1
    fi

    cd "$PROVCHAIN_DIR"

    # Check if already running
    if [ -f "$PROVCHAIN_PID_FILE" ]; then
        PID=$(cat "$PROVCHAIN_PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${YELLOW}ProvChain already running (PID: $PID)${NC}"
            return 1
        else
            echo -e "${YELLOW}Removing stale PID file${NC}"
            rm "$PROVCHAIN_PID_FILE"
        fi
    fi

    # Check if port is already in use
    if lsof -i :$PROVCHAIN_PORT > /dev/null 2>&1; then
        echo -e "${RED}Error: Port $PROVCHAIN_PORT already in use${NC}"
        echo "Process using port:"
        lsof -i :$PROVCHAIN_PORT | grep -v COMMAND
        return 1
    fi

    # Start ProvChain in background with logging
    echo "Starting: cargo run -- web-server --port $PROVCHAIN_PORT"
    nohup cargo run -- web-server --port "$PROVCHAIN_PORT" \
        > "$PROVCHAIN_LOG_FILE" 2>&1 &

    PID=$!
    echo $PID > "$PROVCHAIN_PID_FILE"

    # Wait for service to be ready
    echo -e "${YELLOW}Waiting for ProvChain to start...${NC}"

    # Wait up to 30 seconds for health endpoint
    for i in {1..30}; do
        if curl -s http://localhost:$PROVCHAIN_PORT/health > /dev/null 2>&1; then
            sleep 1  # Give it an extra second to be fully ready
            echo -e "${GREEN}✓ ProvChain started successfully${NC}"
            echo -e "${GREEN}  PID: $PID${NC}"
            echo -e "${GREEN}  Port: $PROVCHAIN_PORT${NC}"
            echo -e "${GREEN}  Log: $PROVCHAIN_LOG_FILE${NC}"

            # Quick health check
            echo ""
            echo "Health status:"
            curl -s http://localhost:$PROVCHAIN_PORT/health | jq -r '.status // "unknown"' 2>/dev/null || echo "checking..."
            return 0
        fi
        echo -n "."
        sleep 1
    done

    echo ""
    echo -e "${RED}✗ ProvChain failed to start within 30 seconds${NC}"
    echo "Last 20 lines of log:"
    tail -20 "$PROVCHAIN_LOG_FILE"
    return 1
}

stop() {
    echo -e "${YELLOW}Stopping native ProvChain...${NC}"

    if [ -f "$PROVCHAIN_PID_FILE" ]; then
        PID=$(cat "$PROVCHAIN_PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "Stopping ProvChain (PID: $PID)..."
            kill "$PID"

            # Wait for process to stop
            for i in {1..10}; do
                if ! ps -p "$PID" > /dev/null 2>&1; then
                    echo -e "${GREEN}✓ ProvChain stopped${NC}"
                    rm "$PROVCHAIN_PID_FILE"
                    return 0
                fi
                sleep 1
            done

            # Force kill if still running
            if ps -p "$PID" > /dev/null 2>&1; then
                echo "Force stopping..."
                kill -9 "$PID"
                sleep 1
            fi

            rm "$PROVCHAIN_PID_FILE"
            echo -e "${GREEN}✓ ProvChain stopped${NC}"
        else
            echo -e "${YELLOW}ProvChain not running (stale PID file)${NC}"
            rm "$PROVCHAIN_PID_FILE"
        fi
    else
        # Try to find ProvChain process by port
        PID=$(lsof -t -i :$PROVCHAIN_PORT 2>/dev/null || true)
        if [ -n "$PID" ]; then
            echo "Found ProvChain on port $PROVCHAIN_PORT (PID: $PID)"
            kill "$PID"
            echo -e "${GREEN}✓ ProvChain stopped${NC}"
        else
            echo -e "${YELLOW}ProvChain not running${NC}"
        fi
    fi
}

status() {
    echo "=== ProvChain Service Status ==="

    if [ -f "$PROVCHAIN_PID_FILE" ]; then
        PID=$(cat "$PROVCHAIN_PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${GREEN}Status: Running${NC}"
            echo "  PID: $PID"
            echo "  Port: $PROVCHAIN_PORT"
            echo ""
            echo "Health check:"
            if curl -s http://localhost:$PROVCHAIN_PORT/health > /dev/null 2>&1; then
                curl -s http://localhost:$PROVCHAIN_PORT/health | jq .
            else
                echo -e "${YELLOW}  Warning: Health endpoint not responding${NC}"
            fi
        else
            echo -e "${YELLOW}Status: Not running (stale PID file)${NC}"
            rm "$PROVCHAIN_PID_FILE"
        fi
    else
        # Check if port is in use
        PID=$(lsof -t -i :$PROVCHAIN_PORT 2>/dev/null || true)
        if [ -n "$PID" ]; then
            echo -e "${YELLOW}Status: Running (no PID file)${NC}"
            echo "  PID: $PID"
            echo "  Port: $PROVCHAIN_PORT"
        else
            echo -e "${YELLOW}Status: Not running${NC}"
        fi
    fi

    echo ""
    echo "Resource usage:"
    ps aux | grep -E "PID|provchain-org|cargo run" | grep -v grep || echo "  No ProvChain process found"
}

health() {
    echo "=== ProvChain Health Check ==="
    echo ""

    # Check if running
    if ! curl -s http://localhost:$PROVCHAIN_PORT/health > /dev/null 2>&1; then
        echo -e "${RED}✗ ProvChain not responding on port $PROVCHAIN_PORT${NC}"
        return 1
    fi

    echo -e "${GREEN}✓ ProvChain is responding${NC}"
    echo ""

    # Health endpoint
    echo "1. Health Endpoint:"
    curl -s http://localhost:$PROVCHAIN_PORT/health | jq .

    echo ""
    echo "2. Blockchain Status:"
    curl -s http://localhost:$PROVCHAIN_PORT/api/blockchain/status | jq . | head -30

    echo ""
    echo "3. Latest Block:"
    curl -s http://localhost:$PROVCHAIN_PORT/api/blocks/latest | jq .

    echo ""
    echo "4. Block Count:"
    COUNT=$(curl -s http://localhost:$PROVCHAIN_PORT/api/blocks/count | jq -r '.count // "error"')
    echo "Total blocks: $COUNT"
}

restart() {
    echo "Restarting ProvChain..."
    stop
    sleep 2
    start
}

logs() {
    if [ -f "$PROVCHAIN_LOG_FILE" ]; then
        echo "=== ProvChain Logs (last 50 lines) ==="
        tail -50 "$PROVCHAIN_LOG_FILE"
    else
        echo "No log file found: $PROVCHAIN_LOG_FILE"
    fi
}

follow_logs() {
    if [ -f "$PROVCHAIN_LOG_FILE" ]; then
        echo "=== Following ProvChain Logs (Ctrl+C to exit) ==="
        tail -f "$PROVCHAIN_LOG_FILE"
    else
        echo "No log file found: $PROVCHAIN_LOG_FILE"
    fi
}

# Show usage if no arguments
usage() {
    echo "Usage: $0 {start|stop|restart|status|health|logs|follow}"
    echo ""
    echo "Commands:"
    echo "  start     - Start native ProvChain service"
    echo "  stop      - Stop native ProvChain service"
    echo "  restart   - Restart ProvChain service"
    echo "  status    - Show service status and health"
    echo "  health    - Run comprehensive health checks"
    echo "  logs      - Show last 50 lines of logs"
    echo "  follow    - Follow logs in real-time"
    echo ""
    echo "Examples:"
    echo "  $0 start    # Start ProvChain"
    echo "  $0 health   # Check service health"
    echo "  $0 status   # Show detailed status"
    echo ""
    echo "Configuration:"
    echo "  ProvChain Directory: $PROVCHAIN_DIR"
    echo "  Port: $PROVCHAIN_PORT"
    echo "  PID File: $PROVCHAIN_PID_FILE"
    echo "  Log File: $PROVCHAIN_LOG_FILE"
}

# Main command dispatcher
case "${1:-}" in
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    status)
        status
        ;;
    health)
        health
        ;;
    logs)
        logs
        ;;
    follow)
        follow_logs
        ;;
    *)
        usage
        exit 1
        ;;
esac
