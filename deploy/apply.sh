#!/bin/sh
# Run as the SSH user in an extracted deployment bundle.
set -eu

task_source=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if ! command -v cargo >/dev/null 2>&1 && [ -r "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi
if ! command -v cargo >/dev/null 2>&1; then
    echo "Cargo is missing on the Pi. Install Rust for the SSH user first." >&2
    exit 1
fi

printf 'Building the Rust server on the Pi...\n'
cargo build --release --locked --manifest-path "$task_source/Cargo.toml" -p dario-server
mkdir -p "$HOME/Development/dario"
printf 'Installing Dario and checking port 3041...\n'
sudo sh "$task_source/deploy/install.sh"
