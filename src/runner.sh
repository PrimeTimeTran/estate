#!/bin/sh

set -u

compile_start=$(date +%s%N)

if /usr/local/bin/language-runner compile; then
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

    if /usr/local/bin/language-runner execute; then
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
  "compile_ms": ${compile_ms},
  "execution_ms": ${execution_ms},
  "exit_code": ${exit_code}
}
EOF

# Keep stdout as the user's program output.
exit "$exit_code"
