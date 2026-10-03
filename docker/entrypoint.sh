#!/bin/sh
set -e

echo "============================================="
echo "  Starting EdgeArena All-In-One Container    "
echo "  - Rust Core Decision Engine: http://0.0.0.0:8080"
echo "  - Next.js Web Dashboard:     http://0.0.0.0:3000"
echo "============================================="

# 1. Start Rust Core Decision API in background
export HOST=0.0.0.0
export PORT=8080
export RUST_LOG=info
/usr/local/bin/edge-api &
API_PID=$!
echo "[EdgeArena] Rust decision engine started (PID: $API_PID)"

# 2. Wait for API to become ready
echo "[EdgeArena] Waiting for decision API readiness..."
for i in $(seq 1 30); do
    if curl -s http://127.0.0.1:8080/health/liveness > /dev/null 2>&1; then
        echo "[EdgeArena] Decision API is HEALTHY on port 8080"
        break
    fi
    sleep 0.5
done

# 3. Start Next.js Frontend Server
echo "[EdgeArena] Starting Next.js Web UI on port 3000..."
cd /app/web
export PORT=3000
export HOSTNAME=0.0.0.0
export NEXT_PUBLIC_API_URL=http://localhost:8080
node server.js &
WEB_PID=$!
echo "[EdgeArena] Next.js Web UI started (PID: $WEB_PID)"

# 4. Trap signals for graceful shutdown
cleanup() {
    echo "[EdgeArena] Shutting down services..."
    kill -TERM "$API_PID" 2>/dev/null || true
    kill -TERM "$WEB_PID" 2>/dev/null || true
    wait "$API_PID" 2>/dev/null || true
    wait "$WEB_PID" 2>/dev/null || true
    echo "[EdgeArena] Clean exit completed."
    exit 0
}

trap cleanup INT TERM

# 5. Monitor child processes
wait -n "$API_PID" "$WEB_PID"
cleanup
