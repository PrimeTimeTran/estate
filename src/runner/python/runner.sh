#!/bin/sh

case "$1" in
    compile)
        ;;

    execute)
        exec python3 /run/solution.py < /run/input
        ;;

    *)
        echo "unknown command: $1" >&2
        exit 1
        ;;
esac
