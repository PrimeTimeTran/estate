#!/usr/bin/env bash
set -euo pipefail

# Resolve project paths.
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ESTATE_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
WORKSPACE_ROOT="$(cd -- "$ESTATE_ROOT/../.." && pwd)"

CARGO_BIN="${CARGO_BIN:-cargo}"
RUN_DIR="$ESTATE_ROOT"

SERVER_PID=""
NATIVE_PID=""

cleanup() {
    local pid

    for pid in "$NATIVE_PID" "$SERVER_PID"; do
        if [[ -n "$pid" ]]; then
            kill "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
        fi
    done

    SERVER_PID=""
    NATIVE_PID=""
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Resolve Cargo's target directory.
HOST="$("$CARGO_BIN" -vV | sed -n 's/^host: //p')"
TARGET="${CARGO_BUILD_TARGET:-$HOST}"
TARGET_DIR="${CARGO_TARGET_DIR:-$WORKSPACE_ROOT/target}"

if [[ "$TARGET_DIR" != /* ]]; then
    TARGET_DIR="$WORKSPACE_ROOT/$TARGET_DIR"
fi

if [[ "$TARGET" == *windows* ]]; then
    EXE=".exe"
else
    EXE=""
fi

if [[ -n "${CARGO_BUILD_TARGET:-}" ]]; then
    DEBUG_DIR="$TARGET_DIR/$TARGET/debug"
else
    DEBUG_DIR="$TARGET_DIR/debug"
fi

build() {
    local bin="$1"

    echo "🦀 Building $bin..."
    (
        cd "$WORKSPACE_ROOT"
        "$CARGO_BIN" build --bin "$bin"
    )
    echo "✅ [$bin] build passed"
}

start() {
    local bin="$1"
    local path="$DEBUG_DIR/$bin$EXE"
    local pid

    if [[ ! -f "$path" ]]; then
        echo "❌ Executable not found: $path" >&2
        exit 1
    fi

    echo "🚀 Starting $bin..."

    (
        cd "$RUN_DIR"
        exec "$path"
    ) &
    pid=$!

    case "$bin" in
        server) SERVER_PID="$pid" ;;
        native) NATIVE_PID="$pid" ;;
    esac

    sleep 2

    if ! kill -0 "$pid" 2>/dev/null; then
        echo "❌ [$bin] failed to start" >&2
        wait "$pid" || true
        exit 1
    fi

    echo "✅ [$bin] startup passed"
}

cd "$WORKSPACE_ROOT"

build server
start server

build native
start native

echo "🛑 Stopping native and server..."
cleanup

build web

echo "🎉 All checks passed"
