#!/usr/bin/env bash
set -e

echo "🦀 Checking server..."
cargo -q build --bin server --no-default-features --features native
echo "✅ [Server] build passed"

echo "🚀 Starting server..."
../../target/debug/server &
SERVER_PID=$!

sleep 2

if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "❌ [Server] failed to start"
    exit 1
fi

echo "✅ [Server] startup passed"


echo "🦀 Checking native..."
cargo -q build --bin native --no-default-features --features native
echo "✅ [Native] build passed"

echo "🚀 Starting native..."
../../target/debug/native &
NATIVE_PID=$!

sleep 2

if ! kill -0 "$NATIVE_PID" 2>/dev/null; then
    echo "❌ [Native] failed to start"
    kill "$SERVER_PID" 2>/dev/null || true
    exit 1
fi

echo "✅ [Native] startup passed"

echo "🛑 Stopping native..."
kill "$NATIVE_PID" 2>/dev/null || true
wait "$NATIVE_PID" 2>/dev/null || true

echo "🛑 Stopping server..."
kill "$SERVER_PID" 2>/dev/null || true
wait "$SERVER_PID" 2>/dev/null || true


echo "🌐 Checking web..."
cargo -q build \
    --bin web \
    --no-default-features \
    --features web \
    --target wasm32-unknown-unknown
echo "✅ [Web] build passed"

echo "🎉 All checks passed"
