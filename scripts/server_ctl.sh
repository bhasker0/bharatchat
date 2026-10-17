#!/usr/bin/env bash
set -eo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_DIR="${REPO_ROOT}/data"
PID_FILE="${DATA_DIR}/gateway.pid"
LOG_FILE="${DATA_DIR}/gateway.log"
GATEWAY_BIN="${REPO_ROOT}/target/release/sovereign-gateway"

mkdir -p "${DATA_DIR}/postgres" "${DATA_DIR}/redis" "${DATA_DIR}/storage"

get_lan_ip() {
    ip -4 addr show scope global | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -n 1 || echo "127.0.0.1"
}

start_db() {
    echo "=== [1/3] Starting Local Sovereign Database Containers ==="
    # 1. PostgreSQL 16
    if podman ps --filter "name=bharatchat-postgres" --format "{{.Names}}" | grep -q "bharatchat-postgres"; then
        echo "      PostgreSQL container 'bharatchat-postgres' is already running."
    else
        echo "      Launching PostgreSQL 16 container 'bharatchat-postgres' on port 5432..."
        podman run -d --name bharatchat-postgres --replace \
            -p 5432:5432 \
            -e POSTGRES_DB=bharatchat \
            -e POSTGRES_USER=bharat_admin \
            -e POSTGRES_PASSWORD=bharat_secure_pass_2026 \
            -v "${DATA_DIR}/postgres:/var/lib/postgresql/data:Z" \
            docker.io/library/postgres:16-alpine >/dev/null
    fi

    # 2. Redis 7
    if podman ps --filter "name=bharatchat-cache" --format "{{.Names}}" | grep -q "bharatchat-cache"; then
        echo "      Redis container 'bharatchat-cache' is already running."
    else
        echo "      Launching Redis 7 container 'bharatchat-cache' on port 6379..."
        podman run -d --name bharatchat-cache --replace \
            -p 6379:6379 \
            -v "${DATA_DIR}/redis:/data:Z" \
            docker.io/library/redis:7-alpine redis-server --appendonly yes >/dev/null
    fi

    # 3. Wait for PostgreSQL
    echo "      Waiting for PostgreSQL to accept connections..."
    for i in {1..20}; do
        if podman exec bharatchat-postgres pg_isready -U bharat_admin -d bharatchat &>/dev/null; then
            break
        fi
        sleep 1
    done

    # 4. Provision Schema
    echo "=== [2/3] Provisioning Database Schema ==="
    python3 "${REPO_ROOT}/scripts/init_local_db.py"
}

start_gateway() {
    echo "=== [3/3] Launching Sovereign Gateway Server ==="
    if [ -f "${PID_FILE}" ]; then
        OLD_PID=$(cat "${PID_FILE}")
        if ps -p "${OLD_PID}" >/dev/null 2>&1; then
            echo "      Gateway server is already running with PID ${OLD_PID}."
            return 0
        fi
        rm -f "${PID_FILE}"
    fi

    if [ ! -f "${GATEWAY_BIN}" ]; then
        echo "      Building release binary..."
        (cd "${REPO_ROOT}" && cargo build --release -p sovereign-gateway)
    fi

    export SOVEREIGN_HOST="0.0.0.0"
    export SOVEREIGN_PORT="8080"
    export DATA_JURISDICTION="INDIA_LOCALIZED_MUMBAI_BLR"

    echo "      Starting daemon listening on 0.0.0.0:8080..."
    setsid "${GATEWAY_BIN}" >> "${LOG_FILE}" 2>&1 &
    NEW_PID=$!
    disown "${NEW_PID}" 2>/dev/null || true
    echo "${NEW_PID}" > "${PID_FILE}"
    echo "      Server process launched with PID ${NEW_PID}."

    # Wait for liveness
    for i in {1..10}; do
        if curl -s http://127.0.0.1:8080/health | grep -q "UP"; then
            echo "      [OK] Server is LIVE and healthy on http://127.0.0.1:8080/health"
            break
        fi
        sleep 1
    done

    LAN_IP=$(get_lan_ip)
    echo "============================================================"
    echo "  BHARATCHAT LOCAL SOVEREIGN SERVER IS RUNNING"
    echo "============================================================"
    echo "  Localhost URL: http://127.0.0.1:8080"
    echo "  Localhost WS : ws://127.0.0.1:8080"
    echo "  LAN Access   : http://${LAN_IP}:8080"
    echo "  LAN WS       : ws://${LAN_IP}:8080"
    echo "  PostgreSQL   : postgresql://bharat_admin:bharat_secure_pass_2026@127.0.0.1:5432/bharatchat"
    echo "  Redis        : redis://127.0.0.1:6379"
    echo "  Media Storage: ${DATA_DIR}/storage"
    echo "  Log File     : ${LOG_FILE}"
    echo "============================================================"
}

stop_all() {
    echo "=== Stopping BharatChat Sovereign Server & Database ==="
    if [ -f "${PID_FILE}" ]; then
        PID=$(cat "${PID_FILE}")
        if ps -p "${PID}" >/dev/null 2>&1; then
            echo "Stopping Gateway daemon (PID ${PID})..."
            kill "${PID}" || true
            sleep 1
        fi
        rm -f "${PID_FILE}"
    fi

    echo "Stopping container 'bharatchat-postgres'..."
    podman stop bharatchat-postgres 2>/dev/null || true
    echo "Stopping container 'bharatchat-cache'..."
    podman stop bharatchat-cache 2>/dev/null || true
    echo "[OK] All local services stopped."
}

show_status() {
    echo "============================================================"
    echo "  BHARATCHAT LOCAL SERVER & DATABASE STATUS"
    echo "============================================================"
    LAN_IP=$(get_lan_ip)
    echo "  Computer Local IP: ${LAN_IP}"
    echo ""
    echo "--- Database Containers ---"
    podman ps --filter "name=bharatchat" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || true
    echo ""
    echo "--- Gateway Server Daemon ---"
    if [ -f "${PID_FILE}" ] && ps -p "$(cat "${PID_FILE}")" >/dev/null 2>&1; then
        echo "  Status : RUNNING (PID $(cat "${PID_FILE}"))"
        echo "  Ports  : 0.0.0.0:8080 (HTTP/1.1 & WebSocket)"
        echo "  Health : $(curl -s http://127.0.0.1:8080/health || echo 'Health check failed')"
    else
        echo "  Status : STOPPED"
    fi
    echo "============================================================"
}

case "${1:-status}" in
    start)
        start_db
        start_gateway
        ;;
    stop)
        stop_all
        ;;
    restart)
        stop_all
        sleep 2
        start_db
        start_gateway
        ;;
    status)
        show_status
        ;;
    init-db)
        python3 "${REPO_ROOT}/scripts/init_local_db.py"
        ;;
    health)
        curl -s http://127.0.0.1:8080/health | python3 -m json.tool || curl -s http://127.0.0.1:8080/health
        ;;
    metrics)
        curl -s http://127.0.0.1:8080/metrics
        ;;
    logs)
        tail -f "${LOG_FILE}"
        ;;
    test)
        echo "Running live verification against http://127.0.0.1:8080..."
        curl -i -s http://127.0.0.1:8080/health
        echo ""
        curl -i -s -X POST http://127.0.0.1:8080/api/v1/identity \
            -H "Content-Type: application/json" \
            -d '{"did":"did:bharat:local_node_owner","vk_hex":"00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff","timestamp":1776000000}'
        echo ""
        curl -i -s http://127.0.0.1:8080/api/v1/identity/did:bharat:local_node_owner
        echo ""
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|init-db|health|metrics|logs|test}"
        exit 1
        ;;
esac
