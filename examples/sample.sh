#!/bin/bash
# Sample shell script for scat syntax highlighting

set -euo pipefail

# Configuration
APP_NAME="scat"
VERSION="1.0.0"
LOG_FILE="/tmp/${APP_NAME}.log"

# Function to display usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  -h, --help    Show this help message"
    echo "  -v, --version Show version"
    echo "  -f, --file    Specify input file"
}

# Function to log messages
log() {
    local level="$1"
    local message="$2"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $message" >> "$LOG_FILE"
}

# Function to process a file
process_file() {
    local file="$1"

    if [[ ! -f "$file" ]]; then
        log "ERROR" "File not found: $file"
        return 1
    fi

    log "INFO" "Processing file: $file"

    while IFS= read -r line; do
        echo "$line"
    done < "$file"
}

# Main logic
main() {
    local file=""

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                usage
                exit 0
                ;;
            -v|--version)
                echo "$APP_NAME v$VERSION"
                exit 0
                ;;
            -f|--file)
                file="$2"
                shift 2
                ;;
            *)
                log "WARN" "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    if [[ -n "$file" ]]; then
        process_file "$file"
    else
        echo "No file specified"
        usage
        exit 1
    fi
}

# Run main function
main "$@"
