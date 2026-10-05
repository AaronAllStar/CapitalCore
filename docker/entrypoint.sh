#!/bin/sh
set -e

echo "============================================="
echo "  Starting CapitalCore Full-Stack Container  "
echo "  - Database:                  PostgreSQL 15 (Port 5432)"
echo "  - Rust Core Decision Engine: http://0.0.0.0:${PORT:-8001}"
echo "  - Next.js Web Dashboard:     http://0.0.0.0:${WEB_PORT:-3001}"
echo "============================================="

# 1. Initialize and start PostgreSQL database service
echo "[CapitalCore] Starting PostgreSQL database..."
if [ ! -d "/var/lib/postgresql/15/main" ]; then
    mkdir -p /var/lib/postgresql/15/main
    chown -R postgres:postgres /var/lib/postgresql
    su - postgres -c "/usr/lib/postgresql/15/bin/initdb -D /var/lib/postgresql/15/main"
fi

service postgresql start || su - postgres -c "/usr/lib/postgresql/15/bin/pg_ctl -D /var/lib/postgresql/15/main -l /var/log/postgresql/server.log start"

# Wait for PostgreSQL readiness
echo "[CapitalCore] Waiting for PostgreSQL readiness..."
for i in $(seq 1 30); do
    if su - postgres -c "pg_isready -h 127.0.0.1 -p 5432" > /dev/null 2>&1; then
        echo "[CapitalCore] PostgreSQL is READY on port 5432"
        break
    fi
    sleep 0.5
done

# Provision user & database
su - postgres -c "psql -c \"CREATE USER capitalcore WITH PASSWORD 'capitalcore' SUPERUSER;\"" 2>/dev/null || true
su - postgres -c "psql -c \"CREATE DATABASE capitalcore OWNER capitalcore;\"" 2>/dev/null || true

export DATABASE_URL=${DATABASE_URL:-"postgres://capitalcore:capitalcore@127.0.0.1:5432/capitalcore"}

# 2. Start Rust Core Decision API in background
export HOST=0.0.0.0
export PORT=${PORT:-8001}
export RUST_LOG=${RUST_LOG:-info}
/usr/local/bin/edge-api &
API_PID=$!
echo "[CapitalCore] Rust decision engine started (PID: $API_PID) on port $PORT"

# 3. Wait for API to become ready
echo "[CapitalCore] Waiting for decision API readiness..."
for i in $(seq 1 30); do
    if curl -s "http://127.0.0.1:${PORT}/health/liveness" > /dev/null 2>&1; then
        echo "[CapitalCore] Decision API is HEALTHY on port $PORT"
        break
    fi
    sleep 0.5
done

# 4. Start Next.js Frontend Server
echo "[CapitalCore] Starting Next.js Web UI on port ${WEB_PORT:-3001}..."
if [ -f "/app/web/apps/web/server.js" ]; then
    cd /app/web/apps/web
else
    cd /app/web
fi
export PORT=${WEB_PORT:-3001}
export HOSTNAME=0.0.0.0
export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-http://localhost:8001}
node server.js &
WEB_PID=$!
echo "[CapitalCore] Next.js Web UI started (PID: $WEB_PID) on port ${PORT}"

# 5. Trap signals for graceful shutdown
cleanup() {
    echo "[CapitalCore] Shutting down services..."
    kill -TERM "$API_PID" 2>/dev/null || true
    kill -TERM "$WEB_PID" 2>/dev/null || true
    service postgresql stop 2>/dev/null || su - postgres -c "/usr/lib/postgresql/15/bin/pg_ctl -D /var/lib/postgresql/15/main stop" 2>/dev/null || true
    wait "$API_PID" 2>/dev/null || true
    wait "$WEB_PID" 2>/dev/null || true
    echo "[CapitalCore] Clean exit completed."
    exit 0
}

trap cleanup INT TERM

# 6. Monitor child processes
wait $API_PID $WEB_PID
cleanup
