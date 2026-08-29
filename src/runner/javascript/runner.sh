#!/bin/sh

case "$1" in
    compile)
        node --check /run/solution.js
        ;;

    execute)
        exec node /run/solution.js
        ;;

    *)
        echo "unknown command: $1" >&2
        exit 1
        ;;
esac
