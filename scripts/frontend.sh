#!/usr/bin/env bash
set -eo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLIENT_DIR="${REPO_ROOT}/client"
DATA_DIR="${REPO_ROOT}/data"
WEB_DIR="${CLIENT_DIR}/build/web"
APK_FILE="${CLIENT_DIR}/build/app/outputs/flutter-apk/app-release.apk"
WEB_PID_FILE="${DATA_DIR}/web.pid"
WEB_LOG_FILE="${DATA_DIR}/web.log"

export PATH="/home/bhasker-savaliya/flutter/bin:/home/bhasker-savaliya/.jdk17/bin:/home/bhasker-savaliya/.android-sdk/platform-tools:${PATH}"
export ANDROID_HOME="${ANDROID_HOME:-/home/bhasker-savaliya/.android-sdk}"
export JAVA_HOME="${JAVA_HOME:-/home/bhasker-savaliya/.jdk17}"

mkdir -p "${DATA_DIR}"

get_lan_ip() {
    ip -4 addr show scope global | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -n 1 || echo "127.0.0.1"
}

is_web_running() {
    if [ -f "${WEB_PID_FILE}" ]; then
        PID=$(cat "${WEB_PID_FILE}")
        if ps -p "${PID}" >/dev/null 2>&1; then
            return 0
        fi
    fi
    return 1
}

build_web() {
    echo "=== [Frontend Tier] Building Flutter Web Client (Release) ==="
    echo "  Note: Database & Backend remain untouched and uninterrupted."
    (cd "${CLIENT_DIR}" && flutter build web --release)
    echo "  [OK] Web bundle successfully built at: ${WEB_DIR}"
}

build_apk() {
    echo "=== [Frontend Tier] Building Flutter Android APK (Release) ==="
    echo "  Note: Database & Backend remain untouched and uninterrupted."
    (cd "${CLIENT_DIR}" && flutter build apk --release)
    echo "  [OK] APK successfully built at: ${APK_FILE}"
    ls -lh "${APK_FILE}"
}

install_apk() {
    echo "=== [Frontend Tier] Installing APK to Connected Android Device via ADB ==="
    echo "  Checking connected devices..."
    adb devices -l
    if ! adb get-state >/dev/null 2>&1; then
        echo "  [ERROR] No Android device connected via ADB or unauthorized."
        echo "  Please connect your device via USB or WiFi debugging and allow access."
        exit 1
    fi

    if [ ! -f "${APK_FILE}" ]; then
        echo "  APK not found. Building APK first..."
        build_apk
    fi

    echo "  Installing ${APK_FILE}..."
    adb install -r "${APK_FILE}"
    echo "  [OK] APK installed successfully on device!"
}

serve_web() {
    echo "=== [Frontend Tier] Starting Sovereign Web Client UI Server ==="
    if is_web_running; then
        echo "  [OK] Web client is already running (PID $(cat "${WEB_PID_FILE}")) on port 3000."
        return 0
    fi

    if [ ! -d "${WEB_DIR}" ]; then
        echo "  Web build directory not found. Building web client first..."
        build_web
    fi

    echo "  Launching HTTP server on 0.0.0.0:3000 (with No-Cache headers)..."
    setsid python3 "${REPO_ROOT}/scripts/serve_web.py" 3000 "${WEB_DIR}" >> "${WEB_LOG_FILE}" 2>&1 &
    NEW_PID=$!
    disown "${NEW_PID}" 2>/dev/null || true
    echo "${NEW_PID}" > "${WEB_PID_FILE}"
    echo "  Web client process launched with PID ${NEW_PID}."

    LAN_IP=$(get_lan_ip)
    echo "  [OK] Web Client is LIVE on:"
    echo "       Local: http://localhost:3000"
    echo "       LAN  : http://${LAN_IP}:3000"
}

stop_web() {
    echo "=== [Frontend Tier] Stopping Sovereign Web Client UI Server ==="
    if is_web_running; then
        PID=$(cat "${WEB_PID_FILE}")
        echo "  Stopping Web Client UI (PID ${PID})..."
        kill "${PID}" || true
        rm -f "${WEB_PID_FILE}"
        echo "  [OK] Web client stopped."
    else
        echo "  Web client is not currently running."
        rm -f "${WEB_PID_FILE}"
    fi
}

restart_web() {
    stop_web
    sleep 1
    serve_web
}

status_frontend() {
    echo "============================================================"
    echo "  BHARATCHAT FRONTEND TIER STATUS"
    echo "============================================================"
    LAN_IP=$(get_lan_ip)
    echo "  LAN IP: ${LAN_IP}"
    echo ""
    echo "--- Web Client Server (Port 3000) ---"
    if is_web_running; then
        echo "  Status : RUNNING (PID $(cat "${WEB_PID_FILE}"))"
        echo "  Local  : http://localhost:3000"
        echo "  LAN    : http://${LAN_IP}:3000"
    else
        echo "  Status : STOPPED"
    fi
    echo ""
    echo "--- Build Artifacts ---"
    if [ -f "${APK_FILE}" ]; then
        echo "  Android APK: Built ($(ls -lh "${APK_FILE}" | awk '{print $5}'))"
        echo "  Path       : ${APK_FILE}"
    else
        echo "  Android APK: Not built yet (run: ./scripts/frontend.sh build-apk)"
    fi

    if [ -d "${WEB_DIR}" ]; then
        echo "  Web Client : Built"
        echo "  Path       : ${WEB_DIR}"
    else
        echo "  Web Client : Not built yet (run: ./scripts/frontend.sh build-web)"
    fi
    echo ""
    echo "--- Connected ADB Devices ---"
    adb devices -l || echo "ADB not available"
    echo "============================================================"
}

test_frontend() {
    echo "=== [Frontend Tier] Running Analyzer and Test Suite ==="
    (cd "${CLIENT_DIR}" && flutter analyze && flutter test)
    echo "  [OK] Frontend tests clean."
}

case "${1:-status}" in
    build-web)
        build_web
        ;;
    build-apk)
        build_apk
        ;;
    install-apk)
        install_apk
        ;;
    serve-web|start-web)
        serve_web
        ;;
    stop-web)
        stop_web
        ;;
    restart-web)
        restart_web
        ;;
    status)
        status_frontend
        ;;
    test)
        test_frontend
        ;;
    *)
        echo "Usage: $0 {build-web|build-apk|install-apk|serve-web|stop-web|restart-web|status|test}"
        exit 1
        ;;
esac
