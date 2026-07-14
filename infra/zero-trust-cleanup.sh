#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# zero-trust-cleanup.sh — ZafafWorld Zero-Trust Production Cleanup
#
# PURPOSE: Remove all "ghost" files from the production deploy directory that
#          could cause stale state, config conflicts, or security risks:
#          - .git history (not needed at runtime, exposes full commit history)
#          - node_modules (rebuilt inside containers, never needed on host)
#          - Build artifacts (dist/, .svelte-kit/, target/ etc.)
#          - Dev-only .env files (only .env.production must exist)
#          - Old logs lingering in the repo
#
# SAFE TO RUN MULTIPLE TIMES. Idempotent.
#
# Usage:
#   chmod +x infra/zero-trust-cleanup.sh
#   bash /opt/zafafworld/infra/zero-trust-cleanup.sh
#
# Requirements:
#   - Run as user 'noon' (not root) OR as root (will warn)
#   - Must be run from /opt/zafafworld or pass DEPLOY_ROOT env var
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

# ── Config ───────────────────────────────────────────────────────────────────
DEPLOY_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"
DRY_RUN="${DRY_RUN:-false}"  # Set DRY_RUN=true to preview without deleting

# ── Colors ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

REMOVED=0
SKIPPED=0
BYTES_FREED=0

# ── Helper: safe_remove ───────────────────────────────────────────────────────
# Removes a path if it exists, logs result, accumulates byte count.
safe_remove() {
    local target="$1"
    local label="${2:-$1}"

    if [[ ! -e "$target" && ! -L "$target" ]]; then
        printf "  ${DIM}─ skip${NC}  %s (not present)\n" "$label"
        SKIPPED=$((SKIPPED + 1))
        return
    fi

    # Measure size before removal
    local size=0
    if command -v du &>/dev/null; then
        size=$(du -sb "$target" 2>/dev/null | awk '{print $1}' || echo 0)
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        printf "  ${YELLOW}~ DRY${NC}   %s (~%s bytes would be removed)\n" "$label" "$size"
        REMOVED=$((REMOVED + 1))
        return
    fi

    rm -rf "$target"
    BYTES_FREED=$((BYTES_FREED + size))
    REMOVED=$((REMOVED + 1))
    printf "  ${GREEN}✓ del${NC}   %s (%s bytes freed)\n" "$label" "$size"
}

# ── Helper: ensure_dir ────────────────────────────────────────────────────────
ensure_dir() {
    local dir="$1"
    local perms="${2:-755}"
    if [[ ! -d "$dir" ]]; then
        mkdir -p "$dir"
        chmod "$perms" "$dir"
        printf "  ${GREEN}✓ create${NC} %s\n" "$dir"
    else
        printf "  ${DIM}─ exists${NC} %s\n" "$dir"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║   ZafafWorld — Zero-Trust Cleanup                           ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
[[ "$DRY_RUN" == "true" ]] && echo -e "${YELLOW}  ⚠ DRY RUN MODE — no files will actually be deleted${NC}\n"

# ── Guard: verify deploy root ─────────────────────────────────────────────────
if [[ ! -d "$DEPLOY_ROOT" ]]; then
    echo -e "${RED}ERROR: DEPLOY_ROOT not found: ${DEPLOY_ROOT}${NC}"
    echo -e "       Run: rsync -av --exclude='.git' ~/zafafworld/ /opt/zafafworld/ first."
    exit 1
fi

if [[ "$(id -u)" -eq 0 ]]; then
    echo -e "${YELLOW}WARNING: Running as root. Files will still be cleaned but ownership${NC}"
    echo -e "${YELLOW}         hardening (Phase 3) should be run as user 'noon'.${NC}\n"
fi

cd "$DEPLOY_ROOT"
echo -e "${CYAN}▸ Working directory: ${DEPLOY_ROOT}${NC}"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# BLOCK 1: Git history — not needed at runtime, exposes commit metadata
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "${BOLD}── Block 1: Git history ────────────────────────────────────────${NC}"
safe_remove "${DEPLOY_ROOT}/.git"          ".git/ (full VCS history)"
safe_remove "${DEPLOY_ROOT}/.gitignore"    ".gitignore"
safe_remove "${DEPLOY_ROOT}/.gitattributes" ".gitattributes"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# BLOCK 2: node_modules — always rebuilt inside containers, never on host
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "${BOLD}── Block 2: node_modules ───────────────────────────────────────${NC}"
for dir in client-web vendor-portal admin-panel; do
    safe_remove "${DEPLOY_ROOT}/${dir}/node_modules" "${dir}/node_modules/"
done
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# BLOCK 3: Build / compile artifacts
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "${BOLD}── Block 3: Build artifacts ────────────────────────────────────${NC}"

# SvelteKit build output
for dir in client-web vendor-portal admin-panel; do
    safe_remove "${DEPLOY_ROOT}/${dir}/.svelte-kit"  "${dir}/.svelte-kit/"
    safe_remove "${DEPLOY_ROOT}/${dir}/build"        "${dir}/build/"
    safe_remove "${DEPLOY_ROOT}/${dir}/dist"         "${dir}/dist/"
    safe_remove "${DEPLOY_ROOT}/${dir}/.vite"        "${dir}/.vite/"
done

# Rust compile cache (target/ can be hundreds of MB)
safe_remove "${DEPLOY_ROOT}/backend-rust/target"     "backend-rust/target/"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# BLOCK 4: Dev-only env files — only .env is allowed
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "${BOLD}── Block 4: Dev/duplicate env files ────────────────────────────${NC}"
safe_remove "${DEPLOY_ROOT}/.env.production"    ".env.production (obsolete production env)"
safe_remove "${DEPLOY_ROOT}/.envx"             ".envx"
safe_remove "${DEPLOY_ROOT}/protect"           "protect (duplicate env)"
safe_remove "${DEPLOY_ROOT}/production.env.example" "production.env.example"

# Guard: make sure .env is still there after cleanup
if [[ ! -f "${DEPLOY_ROOT}/.env" ]]; then
    echo -e "${RED}CRITICAL: .env is missing after cleanup!${NC}"
    echo -e "${RED}          You must restore it before starting the service.${NC}"
    exit 1
fi
echo -e "  ${GREEN}✓ keep${NC}   .env (production secrets — present)"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# BLOCK 5: Old/stale logs
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "${BOLD}── Block 5: Stale logs ──────────────────────────────────────────${NC}"

# Repo-level log files (logs should live in /var/log/zafaf, not the repo)
find "${DEPLOY_ROOT}" -maxdepth 3 -name "*.log" -not -path "*/infra/*" \
    -exec printf "  ${GREEN}✓ del${NC}   %s\n" {} \; \
    -exec rm -f {} \; 2>/dev/null || true

echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# BLOCK 6: Misc redundant files
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "${BOLD}── Block 6: Misc redundant files ───────────────────────────────${NC}"
safe_remove "${DEPLOY_ROOT}/PR_22_VENDOR_MEDIA_AND_LOCATION_CERTIFICATION.md" \
    "PR_22_*.md (dev doc, not needed at runtime)"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# BLOCK 7: Ensure required runtime directories exist
# ═══════════════════════════════════════════════════════════════════════════════
echo -e "${BOLD}── Block 7: Runtime directory setup ────────────────────────────${NC}"
ensure_dir "/var/log/zafaf"       755
ensure_dir "/var/log/zafaf/nginx" 755
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════════════════
MB_FREED=$(echo "scale=2; ${BYTES_FREED}/1048576" | bc 2>/dev/null || echo "?")

echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║                       CLEANUP SUMMARY                       ║${NC}"
echo -e "${BOLD}╠══════════════════════════════════════════════════════════════╣${NC}"
printf  "${BOLD}║${NC}  Items removed:  %-43s${BOLD}║${NC}\n" "$REMOVED"
printf  "${BOLD}║${NC}  Items skipped:  %-43s${BOLD}║${NC}\n" "$SKIPPED"
printf  "${BOLD}║${NC}  Space freed:    %-43s${BOLD}║${NC}\n" "~${MB_FREED} MB"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}DRY RUN complete. Re-run without DRY_RUN=true to apply changes.${NC}"
else
    echo -e "${GREEN}✓ Zero-trust cleanup complete. Proceed to permission hardening.${NC}"
    echo -e "  Next: ${CYAN}bash ${DEPLOY_ROOT}/infra/harden-permissions.sh${NC}"
fi
echo ""
