#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════════
# validate-build.sh — Local pre-flight build check for ZafafWorld frontends
# ══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source config
INFRA_ENV="$SCRIPT_DIR/.env"
if [[ -f "$INFRA_ENV" ]]; then
    set -a; source "$INFRA_ENV"; set +a
fi

# Source logging helper
if [[ -f "$SCRIPT_DIR/lib/logging.sh" ]]; then
    source "$SCRIPT_DIR/lib/logging.sh"
else
    log_info() { echo -e "[INFO] $*"; }
    log_success() { echo -e "\033[0;32m[PASS]\033[0m $*"; }
    log_warn() { echo -e "\033[1;33m[WARN]\033[0m $*"; }
    log_error() { echo -e "\033[0;31m[FAIL]\033[0m $*" >&2; }
fi

REPO_ROOT="${DEPLOY_ROOT:-/opt/zafafworld.net}"
LOG_DIR="${REPO_ROOT}/.build-logs"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"

ALL_TARGETS=(admin-panel client-web vendor-portal)

SKIP_INSTALL=false
TARGETS=()
for arg in "$@"; do
    case "$arg" in
        --skip-install) SKIP_INSTALL=true ;;
        admin-panel|client-web|vendor-portal) TARGETS+=("$arg") ;;
        *) log_error "Unknown argument: $arg" && exit 1 ;;
    esac
done

[[ ${#TARGETS[@]} -eq 0 ]] && TARGETS=("${ALL_TARGETS[@]}")

log_info "══ ZafafWorld Pre-flight Build Validator ══"
log_info "Repo root: $REPO_ROOT"
log_info "Targets  : ${TARGETS[*]}"
echo

check_prerequisites() {
    log_info "Verifying dependencies pre-requisites..."
    local ok=true

    if ! command -v bun &>/dev/null; then
        log_error "bun not found in PATH — install from https://bun.sh"
        ok=false
    else
        log_success "bun found: $(bun --version)"
    fi

    for target in "${TARGETS[@]}"; do
        local dir="${REPO_ROOT}/${target}"
        if [[ ! -d "$dir" ]]; then
            log_error "Directory not found: $dir"
            ok=false
        elif [[ ! -f "${dir}/package.json" ]]; then
            log_error "Missing package.json in: $dir"
            ok=false
        else
            log_success "${target}/ exists"
        fi
    done

    [[ "$ok" == true ]] || { echo; log_error "Pre-requisites checks failed. Aborting."; exit 1; }
    echo
}

build_target() {
    local target="$1"
    local dir="${REPO_ROOT}/${target}"
    local logfile="${LOG_DIR}/${TIMESTAMP}_${target}.log"
    local start_time; start_time=$(date +%s)

    log_info "Building ${target}..."

    mkdir -p "$LOG_DIR"

    if [[ "$SKIP_INSTALL" == false ]]; then
        log_info "  Running bun install..."
        if ! bun install --frozen-lockfile --cwd "$dir" >> "$logfile" 2>&1; then
            log_error "bun install failed — check log: ${logfile}"
            return 1
        fi
        log_success "  Dependencies installed"
    fi

    log_info "  Running svelte-kit sync..."
    bun --cwd "$dir" run svelte-kit sync >> "$logfile" 2>&1 || true

    log_info "  Running svelte-check..."
    if ! bun --cwd "$dir" run svelte-check --threshold error >> "$logfile" 2>&1; then
        local elapsed=$(( $(date +%s) - start_time ))
        log_error "svelte-check failed for $target in ${elapsed}s — check log: ${logfile}"
        echo
        log_warn "Last 20 lines of output:"
        tail -20 "$logfile" | sed 's/^/    /'
        return 1
    fi
    log_success "  Type check passed"

    log_info "  Running bun run build..."
    if ! bun --cwd "$dir" run build >> "$logfile" 2>&1; then
        local elapsed=$(( $(date +%s) - start_time ))
        log_error "Build failed for $target in ${elapsed}s — check log: ${logfile}"
        echo
        log_warn "Last 30 lines of build output:"
        tail -30 "$logfile" | sed 's/^/    /'
        return 1
    fi

    local elapsed=$(( $(date +%s) - start_time ))
    local build_size
    build_size=$(du -sh "${dir}/build" 2>/dev/null | cut -f1 || echo "?")
    log_success "Build passed for $target in ${elapsed}s — output size: ${build_size}"
    return 0
}

check_prerequisites

PASS=()
FAIL=()

for target in "${TARGETS[@]}"; do
    if build_target "$target"; then
        PASS+=("$target")
    else
        FAIL+=("$target")
    fi
    echo
done

echo
log_info "══ Build Report ══"
for t in "${PASS[@]}"; do
    log_success "PASS: ${t}"
done
for t in "${FAIL[@]}"; do
    log_error "FAIL: ${t}"
done

echo
if [[ ${#FAIL[@]} -eq 0 ]]; then
    log_success "All frontends compiled successfully."
    exit 0
else
    log_error "${#FAIL[@]} builds FAILED. Do NOT deploy."
    exit 1
fi
