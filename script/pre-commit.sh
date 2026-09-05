#!/usr/bin/env bash

set -e

start_process() {
    local name="$1"
    shift

    echo "🚀 Starting $name..."
    "$@" &
    LAST_PID=$!

    sleep 2

    if ! kill -0 "$LAST_PID" 2>/dev/null; then
        echo "❌ [$name] failed to start"
        return 1
    fi

    echo "✅ [$name] startup passed"
}

stop_process() {
    local name="$1"
    local pid="$2"

    echo "🛑 Stopping $name..."
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
}

cleanup() {
    [[ -n "${NATIVE_PID:-}" ]] && stop_process "Native" "$NATIVE_PID"
    [[ -n "${SERVER_PID:-}" ]] && stop_process "Server" "$SERVER_PID"
}

trap cleanup EXIT


echo "🦀 Checking native..."
cargo native
echo "✅ [Native] build passed"

echo "🦀 Checking server..."
cargo server
echo "✅ [Server] build passed"

echo "🌐 Checking web..."
cargo web
echo "✅ [Web] build passed"

start_process "Server" cargo server-run
SERVER_PID=$LAST_PID

start_process "Native" cargo native-run
NATIVE_PID=$LAST_PID


# CLI / lifecycle tests go here.
# --------------------------------------------------

# e.g.
# cargo native-run -- start
# cargo native-run -- tray
# cargo native-run -- window
# cargo native-run -- status
# cargo native-run -- doctor

# --------------------------------------------------

echo "🎉 All checks passed"

# #!/usr/bin/env bash
# set -e

# echo "🦀 Checking server..."
# cargo -q build --bin server --no-default-features --features native
# echo "✅ [Server] build passed"

# echo "🚀 Starting server..."
# ../../target/debug/server &
# SERVER_PID=$!

# sleep 2

# if ! kill -0 "$SERVER_PID" 2>/dev/null; then
#     echo "❌ [Server] failed to start"
#     exit 1
# fi

# echo "✅ [Server] startup passed"


# echo "🦀 Checking native..."
# cargo -q build --bin native --no-default-features --features native
# echo "✅ [Native] build passed"

# echo "🚀 Starting native..."
# ../../target/debug/native &
# NATIVE_PID=$!

# sleep 2

# if ! kill -0 "$NATIVE_PID" 2>/dev/null; then
#     echo "❌ [Native] failed to start"
#     kill "$SERVER_PID" 2>/dev/null || true
#     exit 1
# fi

# echo "✅ [Native] startup passed"

# echo "🛑 Stopping native..."
# kill "$NATIVE_PID" 2>/dev/null || true
# wait "$NATIVE_PID" 2>/dev/null || true

# echo "🛑 Stopping server..."
# kill "$SERVER_PID" 2>/dev/null || true
# wait "$SERVER_PID" 2>/dev/null || true


# echo "🌐 Checking web..."
# cargo -q build \
#     --bin web \
#     --no-default-features \
#     --features web \
#     --target wasm32-unknown-unknown
# echo "✅ [Web] build passed"

# echo "🎉 All checks passed"
