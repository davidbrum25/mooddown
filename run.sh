#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
BIN="$DIR/src-tauri/target/release/mooddown"

if [ ! -f "$BIN" ]; then
    BIN="$DIR/src-tauri/target/debug/mooddown"
fi

if [ ! -f "$BIN" ]; then
    echo "Mooddown binary not found. Please build it first with: cd src-tauri && cargo build --release"
    exit 1
fi

export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1

echo "Launching Mooddown daemon..."
exec "$BIN" "$@"
