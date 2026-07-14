#!/bin/sh
set -eu

# ═══════════════════════════════════════════════════════════════════════════════
# entrypoint.sh — Robust Nginx startup with UDS socket readiness gate
# Waits for the Rust backend to create zafaf.sock before starting Nginx,
# preventing 502 errors during cold boot or container restarts.
# ═══════════════════════════════════════════════════════════════════════════════

SOCKET_PATH="/var/run/zafaf/zafaf.sock"
MAX_WAIT=60          # Maximum seconds to wait for socket
POLL_INTERVAL=1      # Seconds between checks

# ── Step 1: Update Cloudflare IP ranges ──────────────────────────────────────
echo "[entrypoint] Running Cloudflare IP compiler..."
/usr/local/bin/update-cloudflare-ips.sh || echo "[entrypoint] Warning: Cloudflare IP fetch failed, using static config."

# ── Step 2: Wait for backend UDS socket to appear ────────────────────────────
echo "[entrypoint] Waiting for backend socket at ${SOCKET_PATH} (timeout: ${MAX_WAIT}s)..."
elapsed=0
while [ ! -S "${SOCKET_PATH}" ]; do
    if [ "${elapsed}" -ge "${MAX_WAIT}" ]; then
        echo "[entrypoint] ERROR: Socket ${SOCKET_PATH} not found after ${MAX_WAIT}s. Starting Nginx anyway (upstream will be unavailable)."
        break
    fi
    sleep "${POLL_INTERVAL}"
    elapsed=$((elapsed + POLL_INTERVAL))
done

if [ -S "${SOCKET_PATH}" ]; then
    echo "[entrypoint] Socket detected after ${elapsed}s. Backend is ready."
fi

# ── Step 2.5: Detect DNS resolver and write to 00-resolver.conf ──────────────
echo "[entrypoint] Detecting DNS resolver..."
DNS_RESOLVER=$(awk '/nameserver/{print $2; exit}' /etc/resolv.conf)
if [ -n "${DNS_RESOLVER}" ]; then
    echo "[entrypoint] Found DNS resolver at ${DNS_RESOLVER}. Writing to /etc/nginx/conf.d/00-resolver.conf..."
    echo "resolver ${DNS_RESOLVER} valid=5s;" > /etc/nginx/conf.d/00-resolver.conf
else
    echo "[entrypoint] WARNING: No DNS resolver found in /etc/resolv.conf. Using default 127.0.0.11."
    echo "resolver 127.0.0.11 valid=5s;" > /etc/nginx/conf.d/00-resolver.conf
fi

# ── Step 3: Start Nginx in foreground ────────────────────────────────────────
echo "[entrypoint] Starting Nginx..."
exec nginx -g "daemon off;"
