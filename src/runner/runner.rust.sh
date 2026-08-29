#!/bin/sh

set -e

compile_start=$(date +%s%N)

rustc /run/solution.rs \
    -O \
    -o /run/solution

compile_end=$(date +%s%N)

compile_ms=$(( (compile_end - compile_start) / 1000000 ))

execution_start=$(date +%s%N)

output=$(/run/solution 2>&1)
exit_code=$?

execution_end=$(date +%s%N)

execution_ms=$(( (execution_end - execution_start) / 1000000 ))

printf '%s\n' "$output"

printf '__RUN_RESULT__%s\n' "$(cat <<EOF
{
  "run_id": "$RUN_ID",
  "compile_ms": $compile_ms,
  "execution_ms": $execution_ms,
  "exit_code": $exit_code
}
EOF
)"
