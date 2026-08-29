#!/bin/sh

case "$1" in
    compile)
        python -m py_compile /run/solution.py
        ;;

    execute)
        exec python /run/solution.py
        ;;

    *)
        echo "unknown command: $1" >&2
        exit 1
        ;;
esac
