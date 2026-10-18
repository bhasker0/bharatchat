#!/usr/bin/env bash
set -eo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_DIR="${REPO_ROOT}/data"
PID_FILE="${DATA_DIR}/gateway.pid"
LOG_FILE="${DATA_DIR}/gateway.log"
GATEWAY_BIN="${REPO_ROOT}/target/release/sovereign-gateway"
PERSISTENCE_DIR="${DATA_DIR}/persistence"

mkdir -p "${DATA_DIR}" "${PERSISTENCE_DIR}"

get_lan_ip() {
    ip -4 addr show scope global | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -n 1 || echo "127.0.0.1"
}

is_gateway_running() {
    if [ -f "${PID_FILE}" ]; then
        PID=$(cat "${PID_FILE}")
        if ps -p "${PID}" >/dev/null 2>&1; then
            return 0
        fi
    fi
    return 1
}

build_backend() {
    echo "=== [Backend Tier] Building Sovereign Gateway Release Binary ==="
    (cd "${REPO_ROOT}" && cargo build --release -p sovereign-gateway)
    echo "  [OK] Release binary compiled at: ${GATEWAY_BIN}"
}

start_backend() {
    echo "=== [Backend Tier] Starting Sovereign Gateway API Server ==="

    if is_gateway_running; then
        echo "  [OK] Gateway daemon is already running (PID $(cat "${PID_FILE}"))."
        return 0
    fi

    if [ ! -f "${GATEWAY_BIN}" ]; then
        echo "  Binary not found. Building release binary..."
        build_backend
    fi

    export SOVEREIGN_HOST="0.0.0.0"
    export SOVEREIGN_PORT="8080"
    export DATA_JURISDICTION="INDIA_LOCALIZED_MUMBAI_BLR"
    export SOVEREIGN_PERSISTENCE_DIR="${PERSISTENCE_DIR}"

    echo "  Starting daemon listening on 0.0.0.0:8080..."
    setsid "${GATEWAY_BIN}" >> "${LOG_FILE}" 2>&1 &
    NEW_PID=$!
    disown "${NEW_PID}" 2>/dev/null || true
    echo "${NEW_PID}" > "${PID_FILE}"
    echo "  Server process launched with PID ${NEW_PID}."

    # Wait for liveness
    for i in {1..10}; do
        if curl -s http://127.0.0.1:8080/health | grep -q "UP"; then
            echo "  [OK] Gateway is LIVE and healthy on http://127.0.0.1:8080/health"
            break
        fi
        sleep 1
    done

    LAN_IP=$(get_lan_ip)
    echo "============================================================"
    echo "  BHARATCHAT BACKEND SERVER RUNNING (Data Persisted)"
    echo "============================================================"
    echo "  Localhost URL: http://127.0.0.1:8080"
    echo "  Localhost WS : ws://127.0.0.1:8080"
    echo "  LAN Access   : http://${LAN_IP}:8080"
    echo "  Persistence  : ${PERSISTENCE_DIR}"
    echo "  Log File     : ${LOG_FILE}"
    echo "============================================================"
}

stop_backend() {
    echo "=== [Backend Tier] Stopping Sovereign Gateway Daemon ==="
    if [ -f "${PID_FILE}" ]; then
        PID=$(cat "${PID_FILE}")
        if ps -p "${PID}" >/dev/null 2>&1; then
            echo "  Stopping process (PID ${PID})..."
            kill "${PID}" || true
            sleep 1
        fi
        rm -f "${PID_FILE}"
    fi
    echo "  [OK] Gateway stopped. Database and stored data remain untouched."
}

restart_backend() {
    echo "=== [Backend Tier] Restarting Sovereign Gateway (Database Untouched) ==="
    stop_backend
    sleep 1
    start_backend
}

show_status() {
    echo "============================================================"
    echo "  BHARATCHAT BACKEND TIER STATUS (Sovereign Gateway)"
    echo "============================================================"
    LAN_IP=$(get_lan_ip)
    if is_gateway_running; then
        PID=$(cat "${PID_FILE}")
        echo "  Status : RUNNING (PID ${PID})"
        echo "  Ports  : 0.0.0.0:8080 (HTTP/1.1 & WebSocket)"
        echo "  LAN URL: http://${LAN_IP}:8080"
        echo "  Health : $(curl -s http://127.0.0.1:8080/health || echo 'Health check failed')"
        echo "  Persist: ${PERSISTENCE_DIR}"
        if [ -d "${PERSISTENCE_DIR}" ]; then
            echo "  Files  :"
            ls -lh "${PERSISTENCE_DIR}"/*.json 2>/dev/null || echo "    (No persisted JSON files yet)"
        fi
    else
        echo "  Status : STOPPED"
    fi
    echo "============================================================"
}

show_logs() {
    tail -f "${LOG_FILE}"
}

case "${1:-status}" in
    start)
        start_backend
        ;;
    stop)
        stop_backend
        ;;
    restart)
        restart_backend
        ;;
    build)
        build_backend
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|build|status|logs}"
        exit 1
        ;;
esac
