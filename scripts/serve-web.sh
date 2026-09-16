#!/bin/sh
set -eu

task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
task_port=${1:-8080}
sh "$task_root/scripts/build-web.sh"
cd "$task_root"
cargo build --release --locked -p dario-server
printf '\nPlay Dario at http://127.0.0.1:%s\nPress Ctrl+C to stop.\n\n' "$task_port"
export DARIO_ADDR="127.0.0.1:$task_port"
exec "$task_root/target/release/dario-server"
