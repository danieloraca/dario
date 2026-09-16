#!/bin/sh
# Build dario-server on the Pi as your normal user, then: sudo sh deploy/install.sh
set -eu

task_source=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
task_unit=/etc/systemd/system/dario.service
task_user=danutz
task_project=/home/danutz/Development/dario
task_web=$task_project/target/web
task_binary=$task_source/target/release/dario-server
if [ "$(id -u)" -ne 0 ]; then
    echo "Run this installer with sudo." >&2
    exit 1
fi
for task_command in systemctl systemd-analyze ss install curl; do
    command -v "$task_command" >/dev/null
done
task_group=$(id -gn "$task_user")
if [ ! -d "$task_project" ]; then
    echo "Create $task_project as $task_user before installing." >&2
    exit 1
fi
for task_file in index.html game.js saves.js game.css favicon.svg dario.wasm vendor/mq_js_bundle.js; do
    test -r "$task_source/target/web/$task_file"
done
test -r "$task_source/deploy/dario.service"
if [ ! -x "$task_binary" ]; then
    echo "Build the server on the Pi first: cargo build --release --locked --manifest-path $task_source/Cargo.toml -p dario-server" >&2
    exit 1
fi

# Only manage this system unit and leave other service locations alone.
task_existing=$(systemctl show dario.service -p FragmentPath --value 2>/dev/null || true)
if [ -n "$task_existing" ] && [ "$task_existing" != "$task_unit" ]; then
    echo "Dario is loaded from $task_existing instead of $task_unit. Inspect it before deploying." >&2
    exit 1
fi
if [ -L "$task_unit" ]; then
    echo "$task_unit is a symlink or masked unit. Inspect it before deploying." >&2
    exit 1
fi
if [ -n "$(ss -H -ltn 'sport = :3041')" ] && ! systemctl is-active --quiet dario.service; then
    echo "Port 3041 is already in use. No services have been changed." >&2
    exit 1
fi
if [ -n "$task_existing" ] && [ "$(systemctl show dario.service -p LoadState --value)" = loaded ]; then
    systemctl stop dario.service
fi
for task_directory in "$task_project/target" "$task_project/target/release" "$task_web"; do
    if [ ! -d "$task_directory" ]; then
        install -d -o "$task_user" -g "$task_group" -m 0755 "$task_directory"
    fi
done
if [ "$task_source" != "$task_project" ]; then
    cp -R "$task_source/target/web/." "$task_web/"
    install -o "$task_user" -g "$task_group" -m 0755 "$task_binary" "$task_project/target/release/dario-server.new"
    mv -f "$task_project/target/release/dario-server.new" "$task_project/target/release/dario-server"
fi
chown -R "$task_user:$task_group" "$task_web"
find "$task_web" -type d -exec chmod 0755 {} +
find "$task_web" -type f -exec chmod 0644 {} +
if [ -f "$task_unit" ] && ! cmp -s "$task_source/deploy/dario.service" "$task_unit"; then
    cp -p "$task_unit" "$task_unit.previous"
fi
install -m 0644 "$task_source/deploy/dario.service" "$task_unit"
systemd-analyze verify "$task_unit"
systemctl daemon-reload
systemctl reset-failed dario.service
systemctl enable --now dario.service
systemctl is-enabled dario.service
systemctl is-active dario.service

task_page=$(curl --fail --silent --show-error --max-time 2 --retry 10 --retry-connrefused --retry-delay 1 http://127.0.0.1:3041/)
case "$task_page" in
    *'<title>Dario'*) ;;
    *) echo "The response is not the Dario page." >&2; exit 1 ;;
esac
task_mime=$(curl --fail --silent --show-error --max-time 2 --head --output /dev/null --write-out '%{content_type}' http://127.0.0.1:3041/dario.wasm)
if [ "$task_mime" != application/wasm ]; then
    echo "Incorrect WASM content type: $task_mime" >&2
    exit 1
fi
echo "Dario is serving on port 3041 and enabled at boot."
