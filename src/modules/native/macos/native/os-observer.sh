#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

swiftc \
  "$SCRIPT_DIR/os-observer.swift" \
  "$SCRIPT_DIR/hid-event-shim.o" \
  -o /tmp/estate-os-observer

exec /tmp/estate-os-observer
