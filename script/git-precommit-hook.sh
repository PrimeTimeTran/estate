
#!/usr/bin/env bash
set -euo pipefail

# CARGO_BIN="${CARGO_BIN:-cargo}"
CARGO_BIN=/mnt/c/Users/seepd/.cargo/bin/cargo.exe \

EXE="${EXE:-}"

SERVER_PID=""
NATIVE_PID=""

cleanup() {
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

build() {
    local bin="$1"

    echo "🦀 Building $bin..."
    "$CARGO_BIN" "$bin"
    echo "✅ [$bin] build passed"
}

start() {
    local bin="$1"
    local path="../../target/debug/${bin}${EXE}"
    local pid

    echo "🚀 Starting $bin..."

    "$path" &
    pid=$!

    if [[ "$bin" == "server" ]]; then
        SERVER_PID="$pid"
    else
        NATIVE_PID="$pid"
    fi

    sleep 2

    if ! kill -0 "$pid" 2>/dev/null; then
        echo "❌ [$bin] failed to start"
        exit 1
    fi

    echo "✅ [$bin] startup passed"
}

# Server
build server
start server

# Native
build native
start native

# Stop both processes
echo "🛑 Stopping native and server..."
cleanup

# Web
build web

echo "🎉 All checks passed"