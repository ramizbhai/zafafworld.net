#!/bin/bash
# ZafafWorld Cloudflare IP Ranges Compiler
# Fetches latest Cloudflare subnets dynamically and builds Nginx config.
set -euo pipefail

OUTPUT_FILE="/etc/nginx/conf.d/00-cloudflare.conf"
TEMP_FILE=$(mktemp)

# Clean up temp file on exit
trap 'rm -f "$TEMP_FILE"' EXIT

echo "# =======================================================================" > "${TEMP_FILE}"
echo "# Dynamic Cloudflare Ingress Subnets (Auto-generated)" >> "${TEMP_FILE}"
echo "# Generated at: $(date -u)" >> "${TEMP_FILE}"
echo "# =======================================================================" >> "${TEMP_FILE}"
echo "" >> "${TEMP_FILE}"

SUCCESS=true

echo "Fetching Cloudflare IPv4 ranges..."
if cf_ipv4=$(curl -s -f https://www.cloudflare.com/ips-v4); then
    while read -r ip; do
        if [[ -n "$ip" ]]; then
            echo "set_real_ip_from ${ip};" >> "${TEMP_FILE}"
        fi
    done <<< "$cf_ipv4"
else
    echo "Warning: Failed to fetch Cloudflare IPv4 ranges." >&2
    SUCCESS=false
fi

echo "Fetching Cloudflare IPv6 ranges..."
if cf_ipv6=$(curl -s -f https://www.cloudflare.com/ips-v6); then
    while read -r ip; do
        if [[ -n "$ip" ]]; then
            echo "set_real_ip_from ${ip};" >> "${TEMP_FILE}"
        fi
    done <<< "$cf_ipv6"
else
    echo "Warning: Failed to fetch Cloudflare IPv6 ranges." >&2
    SUCCESS=false
fi

echo "" >> "${TEMP_FILE}"
echo "real_ip_header CF-Connecting-IP;" >> "${TEMP_FILE}"
echo "real_ip_recursive on;" >> "${TEMP_FILE}"
echo "# Trust internal rootless bridge subnets" >> "${TEMP_FILE}"
echo "set_real_ip_from 10.0.0.0/8;" >> "${TEMP_FILE}"
echo "set_real_ip_from 172.16.0.0/12;" >> "${TEMP_FILE}"
echo "set_real_ip_from 192.168.0.0/16;" >> "${TEMP_FILE}"

if [ "$SUCCESS" = true ]; then
    # Swap to production target location
    mkdir -p "$(dirname "${OUTPUT_FILE}")"
    mv "${TEMP_FILE}" "${OUTPUT_FILE}"
    echo "Nginx Cloudflare Real IP configuration rebuilt successfully at ${OUTPUT_FILE}."
else
    echo "Error: Cloudflare IP fetch failed. Retaining existing configuration at ${OUTPUT_FILE}." >&2
    exit 1
fi
