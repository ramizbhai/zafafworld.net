#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# generate-report.sh — Generates stability report for ZafafWorld production
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source config
INFRA_ENV="$SCRIPT_DIR/../.env"
if [[ -f "$INFRA_ENV" ]]; then
    set -a; source "$INFRA_ENV"; set +a
fi

# Source logging helper
if [[ -f "$SCRIPT_DIR/../lib/logging.sh" ]]; then
    source "$SCRIPT_DIR/../lib/logging.sh"
else
    log_info() { echo -e "[INFO] $*"; }
    log_success() { echo -e "\033[0;32m[PASS]\033[0m $*"; }
    log_warn() { echo -e "\033[1;33m[WARN]\033[0m $*"; }
    log_error() { echo -e "\033[0;31m[FAIL]\033[0m $*" >&2; }
fi

DEPLOY_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"
STATS_LOG="${DEPLOY_ROOT}/infra/scripts/stats-history.log"
NGINX_LOG_DIR="${DEPLOY_ROOT}/infra/nginx/logs"

log_info "══ ZafafWorld Production Stability Report ══"
echo

# 1. client-web Memory Usage Trend
log_info "Analyzing client-web Memory Usage Trend (MB)..."
if [ -f "$STATS_LOG" ] && [ -s "$STATS_LOG" ]; then
    STATS=$(awk '{
        sum += $2;
        count++;
        if (min == "" || $2 < min) min = $2;
        if (max == "" || $2 > max) max = $2;
    } END {
        if (count > 0) {
            printf "Min: %.2f MB | Avg: %.2f MB | Peak: %.2f MB | Datapoints: %d\n", min, sum/count, max, count;
        } else {
            print "No data points collected yet.";
        }
    }' "$STATS_LOG")
    log_success "$STATS"
else
    log_warn "No stats history log found at $STATS_LOG"
    CURR=$(podman stats --no-stream --format "{{.MemUsage}}" zafafworld_client-web_1 2>/dev/null || echo "N/A")
    log_info "Current Instantaneous Memory Usage: $CURR"
fi
echo

# 2. Container Lifecycle & OOM events
log_info "Verifying Container Lifecycle & OOM Events..."
podman ps -a --format "{{.Names}}" | while read -r container; do
    if [ -n "$container" ]; then
        RESTARTS=$(podman inspect "$container" --format '{{.RestartCount}}' 2>/dev/null || echo "0")
        EXIT_CODE=$(podman inspect "$container" --format '{{.State.ExitCode}}' 2>/dev/null || echo "0")
        STATUS=$(podman inspect "$container" --format '{{.State.Status}}' 2>/dev/null || echo "unknown")
        
        if [ "$RESTARTS" -gt 0 ] || [ "$EXIT_CODE" -ne 0 ] || [ "$STATUS" != "running" ]; then
            log_warn "  Container: $container Status: $STATUS Restarts: $RESTARTS ExitCode: $EXIT_CODE"
        else
            log_success "  Container: $container Status: $STATUS Restarts: $RESTARTS"
        fi
    fi
done

OOM_EVENTS=$(journalctl --user --since "48 hours ago" --no-pager 2>/dev/null | grep -iE "oom[-_]killer|killed process|exit code 137" || true)
if [ -n "$OOM_EVENTS" ]; then
    log_error "OOM killer events detected:"
    echo "$OOM_EVENTS" | sed 's/^/    /'
else
    log_success "No OOM-killer events found in journalctl."
fi
echo

# 3. Nginx Upstream & DNS Resolver Analysis
log_info "Scanning Nginx Upstream & DNS Resolver Errors..."
RESTART_TIME="2026/07/07 20:21:00"
if [ -d "$NGINX_LOG_DIR" ] && [ -f "$NGINX_LOG_DIR/error.log" ]; then
    NEW_ERRORS=$(awk -v t="$RESTART_TIME" '$1" "$2 >= t {print}' "$NGINX_LOG_DIR/error.log" | grep -iE "502|504|upstream|resolver|stale|connection refused" || true)
    if [ -n "$NEW_ERRORS" ]; then
        log_warn "Nginx proxy warnings/errors detected since deploy:"
        echo "$NEW_ERRORS" | sed 's/^/    /'
    else
        log_success "No proxy or DNS upstream resolver errors detected since deployment."
    fi
else
    NGINX_CONTAINER=$(podman ps --format '{{.Names}}' 2>/dev/null | grep -i "nginx" | head -1 || true)
    if [ -n "$NGINX_CONTAINER" ]; then
        NEW_ERRORS=$(podman logs "$NGINX_CONTAINER" 2>&1 | grep -iE "502|504|upstream|resolver|stale|connection refused" || true)
        if [ -n "$NEW_ERRORS" ]; then
            log_warn "Nginx container logs proxy warnings:"
            echo "$NEW_ERRORS" | tail -n 10 | sed 's/^/    /'
        else
            log_success "No proxy or DNS resolver errors found in Nginx container logs."
        fi
    fi
fi
echo

# 4. SvelteKit SSR & WP Fetch Errors
log_info "Analyzing SvelteKit SSR & Wordpress Fetch Status..."
CLIENT_CONTAINER=$(podman ps --format '{{.Names}}' 2>/dev/null | grep -i "client-web" | head -1 || true)
if [ -n "$CLIENT_CONTAINER" ]; then
    FETCH_ERRORS=$(podman logs "$CLIENT_CONTAINER" 2>&1 | grep -iE "Fetch timeout|error|failed|504|exception" || true)
    if [ -n "$FETCH_ERRORS" ]; then
        log_warn "SvelteKit fetch timeouts or SSR errors detected:"
        echo "$FETCH_ERRORS" | tail -n 15 | sed 's/^/    /'
    else
        log_success "No SvelteKit fetch timeouts or SSR errors detected in logs."
    fi
else
    log_warn "client-web container is offline."
fi
echo

# 5. SvelteKit SSR Response Time Metrics
log_info "Calculating SvelteKit SSR Response Time Metrics..."
ACCESS_LOG="${NGINX_LOG_DIR}/access.log"
if [ -f "$ACCESS_LOG" ]; then
    MONTHS=(X Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec)
    LOG_START_TIME=$(echo "$RESTART_TIME" | awk -F'[/ ]' -v months="${MONTHS[*]}" '
        BEGIN { split(months, m, " ") }
        { printf "%02d/%s/%d:%s\n", $3, m[$2+1], $1, $4 }
    ')
    
    SSR_STATS=$(awk -v start="$LOG_START_TIME" '
        function get_ts(str) {
            gsub(/^\[/, "", str);
            return str;
        }
        {
            ts = get_ts($4);
            if (ts >= start) {
                match($0, /rt=[0-9.]+/);
                if (RSTART > 0) {
                    rt_val = substr($0, RSTART+3, RLENGTH-3) + 0;
                    if ($7 !~ /\.(js|css|png|jpg|jpeg|gif|svg|ico|webp|woff2|txt|xml|php)/ && $7 !~ /^\/(api|bff|assets)\// && $7 !~ /healthz/ && $9 ~ /^(200|302|307)$/) {
                        sum += rt_val;
                        count++;
                        if (max == "" || rt_val > max) {
                            max = rt_val;
                            max_req = $7;
                        }
                        if (min == "" || rt_val < min) min = rt_val;
                    }
                }
            }
        }
        END {
            if (count > 0) {
                printf "Average SSR Response Time: %.1f ms (%.3fs)\nSlowest SSR Response Time:  %.1f ms (%.3fs) on %s\nTotal Measured Page Views:  %d\n", 
                    (sum/count)*1000, sum/count, max*1000, max, max_req, count;
            } else {
                print "No page views recorded since final deployment.";
            }
        }
    ' "$ACCESS_LOG" || echo "  Error parsing access logs")
    log_success "$SSR_STATS"
else
    log_warn "Nginx access.log not found."
fi
