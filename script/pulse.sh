#!/usr/bin/env bash
set -e

echo "🦀 Checking native..."
cargo native
echo "✅ [Native] build passed"

echo "🦀 Checking server..."
cargo server
echo "✅ [Server] build passed"

echo "🌐 Checking web..."
cargo web
echo "✅ [Web] build passed"

echo "🎉 All checks passed"
