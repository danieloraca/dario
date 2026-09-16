#!/bin/sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
    echo "This bundle script is for macOS. Use cargo run --release on other systems." >&2
    exit 1
fi

task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$task_root"
cargo build --release --locked
task_bundle="$task_root/target/Dario.app"
mkdir -p "$task_bundle/Contents/MacOS"
cp target/release/dario "$task_bundle/Contents/MacOS/dario.new"
mv "$task_bundle/Contents/MacOS/dario.new" "$task_bundle/Contents/MacOS/dario"
cat > "$task_bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
    <key>CFBundleName</key><string>Dario</string>
    <key>CFBundleDisplayName</key><string>Dario</string>
    <key>CFBundleIdentifier</key><string>local.dario.game</string>
    <key>CFBundleExecutable</key><string>dario</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>0.1.0</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
plutil -lint "$task_bundle/Contents/Info.plist"
# A local ad-hoc signature needs no Apple account or distribution credentials.
codesign --force --sign - "$task_bundle"
codesign --verify --strict "$task_bundle"
printf 'Ready to play: %s\n' "$task_bundle"
