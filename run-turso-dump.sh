#!/bin/bash

# Exit immediately on error
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

DB_PATH=""

while [ $# -gt 0 ]; do
    case "$1" in
        --db)
            DB_PATH="$2"
            shift 2
            ;;
        *)
            echo "unknown argument: $1"
            exit 1
            ;;
    esac
done

echo "Stopping existing turso-dump processes..."
pkill -f "target/release/turso-dump" 2>/dev/null || true

echo "Building turso-dump..."
(cd "$SCRIPT_DIR" && cargo build --release)

echo "Starting turso-dump..."
if [ -n "$DB_PATH" ]; then
    "$SCRIPT_DIR/target/release/turso-dump" "$DB_PATH"
else
    "$SCRIPT_DIR/target/release/turso-dump"
fi
