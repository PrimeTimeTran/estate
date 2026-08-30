#!/bin/sh

set -u

setup_ms=0

case "${LANGUAGE:-}" in
    rust)
        LANGUAGE_RUNNER="/usr/local/bin/runner.rust.sh"
        ;;
    python)
        LANGUAGE_RUNNER="/usr/local/bin/runner.python.sh"
        ;;
    javascript)
        LANGUAGE_RUNNER="/usr/local/bin/runner.javascript.sh"
        ;;
    *)
        echo "unsupported language: ${LANGUAGE:-<unset>}" >&2
        exit 2
        ;;
esac

compile_start=$(date +%s%N)

if "$LANGUAGE_RUNNER" compile; then
    compile_exit=0
else
    compile_exit=$?
fi

compile_end=$(date +%s%N)
compile_ms=$(( (compile_end - compile_start) / 1000000 ))

if [ "$compile_exit" -ne 0 ]; then
    execution_ms=0
    exit_code=$compile_exit
else
    execution_start=$(date +%s%N)

    if "$LANGUAGE_RUNNER" execute; then
        exit_code=0
    else
        exit_code=$?
    fi

    execution_end=$(date +%s%N)
    execution_ms=$(( (execution_end - execution_start) / 1000000 ))
fi

cat > /run/result.json <<EOF
{
  "run_id": "${RUN_ID}",
  "setup_ms": ${setup_ms},
  "compile_ms": ${compile_ms},
  "execution_ms": ${execution_ms},
  "exit_code": ${exit_code}
}
EOF

exit "$exit_code"
