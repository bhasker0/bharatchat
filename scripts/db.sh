#!/usr/bin/env bash
set -eo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_DIR="${REPO_ROOT}/data"
POSTGRES_DIR="${DATA_DIR}/postgres"
REDIS_DIR="${DATA_DIR}/redis"

mkdir -p "${POSTGRES_DIR}" "${REDIS_DIR}"

is_postgres_running() {
    podman ps --filter "name=bharatchat-postgres" --format "{{.Names}}" | grep -q "bharatchat-postgres"
}

is_redis_running() {
    podman ps --filter "name=bharatchat-cache" --format "{{.Names}}" | grep -q "bharatchat-cache"
}

start_db() {
    echo "=== [Database Tier] Starting Local Sovereign Database Containers ==="

    if is_postgres_running; then
        echo "  [OK] PostgreSQL container 'bharatchat-postgres' is already running on port 5432."
    elif podman ps -a --format "{{.Names}}" | grep -q "bharatchat-postgres"; then
        echo "  Restarting existing PostgreSQL container 'bharatchat-postgres'..."
        podman start bharatchat-postgres >/dev/null
    else
        echo "  Launching PostgreSQL 16 container 'bharatchat-postgres' on port 5432..."
        podman run -d --name bharatchat-postgres \
            -p 5432:5432 \
            -e POSTGRES_DB=bharatchat \
            -e POSTGRES_USER=bharat_admin \
            -e POSTGRES_PASSWORD=bharat_secure_pass_2026 \
            -v "${POSTGRES_DIR}:/var/lib/postgresql/data:Z" \
            docker.io/library/postgres:16-alpine >/dev/null
    fi

    if is_redis_running; then
        echo "  [OK] Redis container 'bharatchat-cache' is already running on port 6379."
    elif podman ps -a --format "{{.Names}}" | grep -q "bharatchat-cache"; then
        echo "  Restarting existing Redis container 'bharatchat-cache'..."
        podman start bharatchat-cache >/dev/null
    else
        echo "  Launching Redis 7 container 'bharatchat-cache' on port 6379..."
        podman run -d --name bharatchat-cache \
            -p 6379:6379 \
            -v "${REDIS_DIR}:/data:Z" \
            docker.io/library/redis:7-alpine redis-server --appendonly yes >/dev/null
    fi

    # Wait for PostgreSQL
    echo "  Waiting for PostgreSQL readiness..."
    for i in {1..20}; do
        if podman exec bharatchat-postgres pg_isready -U bharat_admin -d bharatchat &>/dev/null; then
            break
        fi
        sleep 1
    done

    # Run non-destructive schema migrations
    python3 "${REPO_ROOT}/scripts/init_local_db.py"
    echo "=== [Database Tier] Database service is READY and persistent ==="
}

stop_db() {
    echo "=== [Database Tier] Stopping Local Sovereign Database Containers ==="
    if is_postgres_running; then
        echo "  Stopping 'bharatchat-postgres'..."
        podman stop bharatchat-postgres >/dev/null || true
    fi
    if is_redis_running; then
        echo "  Stopping 'bharatchat-cache'..."
        podman stop bharatchat-cache >/dev/null || true
    fi
    echo "  [OK] Database containers stopped. Data volumes in data/postgres remain preserved."
}

restart_db() {
    echo "=== [Database Tier] Restarting Database Containers (Non-Destructive) ==="
    stop_db
    sleep 1
    start_db
}

show_status() {
    echo "============================================================"
    echo "  BHARATCHAT DATABASE TIER STATUS (PostgreSQL 16 & Redis 7)"
    echo "============================================================"
    echo "  Persistent Storage Path: ${POSTGRES_DIR}"
    echo ""
    echo "--- Active Podman Containers ---"
    podman ps --filter "name=bharatchat" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || true
    echo ""
    if is_postgres_running; then
        echo "--- Relational Database Table Metrics ---"
        podman exec -i bharatchat-postgres psql -U bharat_admin -d bharatchat -c "
            SELECT 
                relname AS table_name, 
                n_live_tup AS estimated_rows
            FROM pg_stat_user_tables
            ORDER BY relname;
        "
    else
        echo "  PostgreSQL is NOT currently running."
    fi
    echo "============================================================"
}

show_rows() {
    if is_postgres_running; then
        echo "--- Table Counts in Database 'bharatchat' ---"
        podman exec -i bharatchat-postgres psql -U bharat_admin -d bharatchat -c "
            SELECT 'identities' AS table, count(*) AS total_records FROM identities
            UNION ALL
            SELECT 'chat_messages', count(*) FROM chat_messages
            UNION ALL
            SELECT 'call_logs', count(*) FROM call_logs
            UNION ALL
            SELECT 'broadcast_channels', count(*) FROM broadcast_channels
            UNION ALL
            SELECT 'commerce_orders', count(*) FROM commerce_orders;
        "
    else
        echo "PostgreSQL is not running."
        exit 1
    fi
}

open_psql() {
    if ! is_postgres_running; then
        echo "Error: Database is not running. Run './scripts/db.sh start' first."
        exit 1
    fi
    podman exec -it bharatchat-postgres psql -U bharat_admin -d bharatchat "$@"
}

case "${1:-status}" in
    start)
        start_db
        ;;
    stop)
        stop_db
        ;;
    restart)
        restart_db
        ;;
    status)
        show_status
        ;;
    rows)
        show_rows
        ;;
    psql)
        shift
        open_psql "$@"
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|rows|psql}"
        exit 1
        ;;
esac
