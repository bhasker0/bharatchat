#!/usr/bin/env bash
set -eo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SCRIPT_DB="${REPO_ROOT}/scripts/db.sh"
SCRIPT_BACKEND="${REPO_ROOT}/scripts/backend.sh"
SCRIPT_FRONTEND="${REPO_ROOT}/scripts/frontend.sh"

chmod +x "${SCRIPT_DB}" "${SCRIPT_BACKEND}" "${SCRIPT_FRONTEND}" 2>/dev/null || true

case "${1:-status}" in
    start)
        echo "=== [Master Orchestrator] Starting BharatChat Sovereign Stack ==="
        "${SCRIPT_DB}" start
        "${SCRIPT_BACKEND}" start
        "${SCRIPT_FRONTEND}" serve-web
        echo ""
        echo "============================================================"
        echo "  BHARATCHAT THREE-TIER STACK IS LIVE"
        echo "  - Tier 1 (Database): PostgreSQL :5432 & Redis :6379 (Persistent)"
        echo "  - Tier 2 (Backend) : Sovereign Gateway :8080 (Data-Preserved)"
        echo "  - Tier 3 (Frontend): Web UI :3000 & Android Client"
        echo "============================================================"
        ;;
    stop)
        echo "=== [Master Orchestrator] Stopping Frontend & Backend (DB stays preserved) ==="
        "${SCRIPT_FRONTEND}" stop-web || true
        "${SCRIPT_BACKEND}" stop || true
        echo "  [INFO] Database containers & persistent storage remain safe and running."
        echo "         To stop the database containers as well, run: ./scripts/server_ctl.sh stop-all"
        ;;
    stop-all)
        echo "=== [Master Orchestrator] Stopping All Services ==="
        "${SCRIPT_FRONTEND}" stop-web || true
        "${SCRIPT_BACKEND}" stop || true
        "${SCRIPT_DB}" stop || true
        echo "  [OK] All services stopped. Data remains safely preserved on disk."
        ;;
    restart)
        echo "=== [Master Orchestrator] Restarting Backend & Frontend (DB zero-downtime) ==="
        "${SCRIPT_BACKEND}" restart
        "${SCRIPT_FRONTEND}" restart-web
        ;;
    status)
        echo "============================================================"
        echo "  BHARATCHAT THREE-TIER ARCHITECTURE STATUS"
        echo "============================================================"
        "${SCRIPT_DB}" status
        echo ""
        "${SCRIPT_BACKEND}" status
        echo ""
        "${SCRIPT_FRONTEND}" status
        ;;
    build-web)
        "${SCRIPT_FRONTEND}" build-web
        ;;
    build-apk)
        "${SCRIPT_FRONTEND}" build-apk
        ;;
    install-apk)
        "${SCRIPT_FRONTEND}" install-apk
        ;;
    db-rows)
        "${SCRIPT_DB}" rows
        ;;
    psql)
        "${SCRIPT_DB}" psql
        ;;
    health)
        "${SCRIPT_BACKEND}" health
        ;;
    metrics)
        "${SCRIPT_BACKEND}" metrics
        ;;
    logs)
        "${SCRIPT_BACKEND}" logs
        ;;
    test)
        echo "=== Running Sovereign Stack Verification ==="
        "${SCRIPT_BACKEND}" test
        echo ""
        "${SCRIPT_DB}" rows
        ;;
    *)
        echo "Usage: $0 {start|stop|stop-all|restart|status|build-web|build-apk|install-apk|db-rows|psql|health|metrics|logs|test}"
        exit 1
        ;;
esac
