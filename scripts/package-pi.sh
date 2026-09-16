#!/bin/sh
set -eu

task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sh "$task_root/scripts/build-web.sh"
tar -czf "$task_root/target/dario-pi.tar.gz" \
    -C "$task_root" Makefile Cargo.toml Cargo.lock build.rs README.md src server progress web scripts deploy target/web
printf 'Pi deployment archive: %s/target/dario-pi.tar.gz\n' "$task_root"
