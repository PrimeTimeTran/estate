#!/bin/sh

case "$1" in
    compile)
        rustc /run/solution.rs -O -o /run/solution
        ;;
    execute)
        exec /run/solution < /run/input
        ;;
    *)
        echo "unknown command: $1" >&2
        exit 1
        ;;
esac
