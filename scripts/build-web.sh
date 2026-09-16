#!/bin/sh
set -eu

task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$task_root"
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
    echo "Install the browser target first: rustup target add wasm32-unknown-unknown" >&2
    exit 1
fi

cargo build --release --locked --target wasm32-unknown-unknown
mkdir -p target/web/vendor
cp target/wasm32-unknown-unknown/release/dario.wasm target/web/dario.wasm
cp web/index.html web/game.css web/game.js web/saves.js web/favicon.svg target/web/

# Assemble only the graphics and audio plugins from their locked Rust dependencies.
# Macroquad 0.4.16's prebuilt bundle includes an unused plugin with a JS error.
cargo metadata --format-version 1 --locked --offline --filter-platform wasm32-unknown-unknown |
python3 -c '
import json, pathlib, shutil, sys
packages = {package["name"]: package for package in json.load(sys.stdin)["packages"]}
graphics = pathlib.Path(packages["miniquad"]["manifest_path"]).parent
audio = pathlib.Path(packages["quad-snd"]["manifest_path"]).parent
output = pathlib.Path("target/web/vendor")
runtime = (graphics / "js/gl.js").read_text()
runtime += "\n;(function () {\n" + (audio / "js/audio.js").read_text() + "\n})();\n"
(output / "mq_js_bundle.js").write_text(runtime)
for name in ("LICENSE-MIT", "LICENSE-APACHE"):
    shutil.copy2(graphics / name, output / name)
notices = []
for name in ("miniquad", "quad-snd"):
    package = packages[name]
    notices.append("{} {}\nAuthors: {}\nLicense: {}\n".format(name, package["version"], ", ".join(package["authors"]), package["license"]))
(output / "NOTICE.txt").write_text("\n".join(notices))
'
printf 'Browser build ready: %s/target/web\n' "$task_root"
