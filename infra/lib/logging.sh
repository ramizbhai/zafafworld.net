#!/usr/bin/env bash
# ZafafWorld Shared Infrastructure Logging Library & Config Parser
# Sourced by other shell scripts for consistent, colored, timestamped output and safe env loading.

# Color Codes
LOG_RED='\033[0;31m'
LOG_GRN='\033[0;32m'
LOG_YLW='\033[1;33m'
LOG_CYN='\033[0;36m'
LOG_RST='\033[0m'

# Check if stdout is a TTY to toggle colors
if [[ ! -t 1 ]]; then
    LOG_RED=""
    LOG_GRN=""
    LOG_YLW=""
    LOG_CYN=""
    LOG_RST=""
fi

log_info() {
    echo -e "${LOG_CYN}[INFO] [$(date '+%Y-%m-%d %H:%M:%S')]${LOG_RST} $*"
}

log_success() {
    echo -e "${LOG_GRN}[PASS] [$(date '+%Y-%m-%d %H:%M:%S')]${LOG_RST} $*"
}

log_warn() {
    echo -e "${LOG_YLW}[WARN] [$(date '+%Y-%m-%d %H:%M:%S')]${LOG_RST} $*"
}

log_error() {
    echo -e "${LOG_RED}[FAIL] [$(date '+%Y-%m-%d %H:%M:%S')]${LOG_RST} $*" >&2
}

# Robust configuration file parser to prevent bash command evaluation of unquoted values
load_env() {
    local env_file="$1"
    if [[ -f "$env_file" ]]; then
        while IFS= read -r line || [[ -n "$line" ]]; do
            # Trim leading/trailing whitespace
            line=$(echo "$line" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
            # Ignore comments and empty lines
            [[ -z "$line" || "$line" =~ ^# ]] && continue
            
            if [[ "$line" =~ ^([^=]+)=(.*)$ ]]; then
                local key="${BASH_REMATCH[1]}"
                local val="${BASH_REMATCH[2]}"
                # Trim key and val
                key=$(echo "$key" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
                val=$(echo "$val" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
                # Strip surrounding single or double quotes
                val="${val#\"}"
                val="${val%\"}"
                val="${val#\'}"
                val="${val%\'}"
                export "$key"="$val"
            fi
        done < "$env_file"
    fi
}
