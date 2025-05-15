#!/usr/bin/env bash

set -euo pipefail

if (( $# != 1 )); then
  echo "Usage: $0 <action>  # action: execute, setup, setup_no_write, prove, prove_no_write, or verify" >&2
  exit 1
fi

action="$1"
[[ $action == --* ]] || action="--$action"

cargo build --release

RUST_LOG=info ../target/release/sha "$action" &
PID=$!

echo "PID: $PID"

while kill -0 "$PID" 2>/dev/null; do
  ts=$(date +%s)
  echo "===== Snapshot at $ts ====="
  vmmap -summary "$PID" | grep "footprint"
  sleep 0.1
done
