#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# infra/sync-media.sh — Wrapper for sync-media.py
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "$SCRIPT_DIR/sync-media.py" "$@"
