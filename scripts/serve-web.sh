#!/bin/sh
set -eu

task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
task_port=${1:-8080}
sh "$task_root/scripts/build-web.sh"
printf '\nPlay Dario at http://127.0.0.1:%s\nPress Ctrl+C to stop.\n\n' "$task_port"
exec python3 -m http.server "$task_port" --bind 127.0.0.1 --directory "$task_root/target/web"
