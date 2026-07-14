#!/bin/sh
set -eu

# ── Detect DNS resolver and write to 00-resolver.conf ──────────────────────
echo "[entrypoint] Detecting DNS resolver..."
DNS_RESOLVER=$(awk '/nameserver/{print $2; exit}' /etc/resolv.conf)
if [ -n "${DNS_RESOLVER}" ]; then
    echo "[entrypoint] Found DNS resolver at ${DNS_RESOLVER}. Writing to /etc/nginx/conf.d/00-resolver.conf..."
    echo "resolver ${DNS_RESOLVER} valid=10s;" > /etc/nginx/conf.d/00-resolver.conf
else
    echo "[entrypoint] WARNING: No DNS resolver found in /etc/resolv.conf. Using default 127.0.0.11."
    echo "resolver 127.0.0.11 valid=10s;" > /etc/nginx/conf.d/00-resolver.conf
fi

# ── Start Nginx in foreground ──────────────────────────────────────────────
echo "[entrypoint] Starting Nginx..."
exec nginx -g "daemon off;"
