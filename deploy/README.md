# Raspberry Pi service

Dario follows the existing Solitaire service layout: a Rust binary in `~/Development/dario/target/release`, a listening address in an environment variable, and a systemd unit in `/etc/systemd/system`.

The service runs **`dario-server`** and serves the prebuilt browser game from `~/Development/dario/target/web` on port **3041**. The browser runs the Rust game as WebAssembly. The Pi server requires no display, audio libraries, or Python runtime. Building the server on the Pi needs Rust 1.85 or newer; the installer also uses `curl` for its health check.

## Deploy with one command

From your Mac's Dario project, run:

```sh
make deploy
```

The command builds the browser game locally, connects to `danutz@192.168.0.25`, uploads to a temporary directory, and builds `dario-server` on the Pi as `danutz`. It then installs the files, restarts only `dario.service`, enables boot startup, and checks the page and WASM content type on port 3041. SSH reuses one authenticated connection; enter the SSH and sudo passwords in your terminal when prompted. No password is saved by the scripts.

The running service stays up during the builds and upload. A failed build or transfer stops deployment before installation. Temporary deployment files and the SSH connection are cleaned up afterward. Files installed into `~/Development/dario/target` stay in place; project source files on the Pi are preserved.

The Mac needs the existing browser-build tools (Rust with `wasm32-unknown-unknown`, Python 3), `make`, SSH, and SCP. The Pi needs Cargo and permission for `danutz` to use sudo. If Cargo is outside the SSH command's PATH, the deployment loads `~/.cargo/env` when available.

To change the Pi's SSH destination while retaining the configured account and paths:

```sh
make deploy PI_HOST=danutz@new-pi-address
```

The installer manages `/etc/systemd/system/dario.service`. When its contents change, the previous unit is saved as `dario.service.previous` before replacement. A masked or symlinked unit, a unit loaded from another location, or an occupied port belonging to another service stops installation for inspection.

## Service file

Use [dario.service](dario.service):

```ini
[Unit]
Description=Dario web app
After=network.target

[Service]
Type=simple
User=danutz
WorkingDirectory=/home/danutz/Development/dario
ExecStart=/home/danutz/Development/dario/target/release/dario-server
Restart=on-failure
RestartSec=2
Environment=DARIO_ADDR=0.0.0.0:3041

[Install]
WantedBy=multi-user.target
```

The `dario` binary opens the desktop game; use `dario-server` for this service. The server defaults to `127.0.0.1:3041` outside systemd. `DARIO_ADDR` overrides that address. It serves only the known public game files and checks that they exist before listening.

You can keep an editable service file in `~/Development/dario/dario.service`, then install a copy in `/etc/systemd/system`. Use absolute paths inside the unit; `~` is not expanded there.

## Build and upload from the Mac

From the Mac's Dario project:

```sh
sh scripts/package-pi.sh
scp target/dario-pi.tar.gz danutz@192.168.0.25:dario-pi.tar.gz
```

This archive contains the source, deployment scripts, and the complete prebuilt `target/web/`. It omits native binaries, which must be built for the Pi's Linux architecture. If you uploaded an older Python-server bundle, upload this rebuilt archive again.

In your existing Pi SSH session, extract to a temporary folder and build **as your normal user**:

```sh
mkdir -p "$HOME/Development/dario"
task_stage=$(mktemp -d)
tar -xzf "$HOME/dario-pi.tar.gz" -C "$task_stage"
cargo build --release --locked --manifest-path "$task_stage/Cargo.toml" -p dario-server
```

The server package has no dependency on Macroquad. This command does not compile the desktop game or require ALSA/X11 development packages.

Then install. An existing regular `/etc/systemd/system/dario.service`, including the earlier Python version or a blank file, is updated to the bundled unit:

```sh
sudo sh "$task_stage/deploy/install.sh"
```

Enter SSH and sudo passwords only in the terminal. The installer checks the port, briefly stops only an existing Dario service, copies the built server and browser files into the project folder, installs the unit, and enables it at boot. It checks the HTTP page and WASM content type before reporting success. It does not replace your project source files.

The important installed paths are:

```text
/home/danutz/Development/dario/target/release/dario-server
/home/danutz/Development/dario/target/web/index.html
/home/danutz/Development/dario/target/web/dario.wasm
/home/danutz/Development/dario/target/web/game.js
/home/danutz/Development/dario/target/web/game.css
/home/danutz/Development/dario/target/web/favicon.svg
/home/danutz/Development/dario/target/web/vendor/
/etc/systemd/system/dario.service
```

The `target/web/vendor/` folder includes the JavaScript runtime and its license notices. The source `web/` folder alone is not a runnable build: it lacks the compiled WASM and runtime.

## Build directly in the Pi checkout

If the Pi already has the current source checkout, you may build everything there instead. Python 3 is used only by the browser build script to assemble its JavaScript runtime:

```sh
cd ~/Development/dario
rustup target add wasm32-unknown-unknown
sh scripts/build-web.sh
cargo build --release --locked -p dario-server
```

`sudo sh deploy/install.sh` installs and starts it. Or install manually:

```sh
sudo install -m 0644 deploy/dario.service /etc/systemd/system/dario.service
sudo systemd-analyze verify /etc/systemd/system/dario.service
sudo systemctl daemon-reload
sudo systemctl reset-failed dario.service
sudo systemctl enable dario.service
sudo systemctl restart dario.service
```

If you edited `~/Development/dario/dario.service`, use that file as the source for `install` instead of `deploy/dario.service`.

## Verify

```sh
systemctl is-enabled dario.service
systemctl is-active dario.service
systemctl status dario.service --no-pager
curl -I http://127.0.0.1:3041/
curl -I http://127.0.0.1:3041/dario.wasm
journalctl -u dario.service -n 50 --no-pager
```

The service should be `enabled` and `active`; the WASM response should have `Content-Type: application/wasm`. Open [Dario on the Pi](http://192.168.0.25:3041/) from another device on the same network. If the Pi's address changes, use the new address or reserve it in your router.

Enabling the service registers startup on future boots. `Restart=on-failure` restarts the server after a crash with a two-second delay. Closing SSH does not stop a system service. These checks do not reboot the Pi or interrupt its other services; a full reboot test is separate.

If startup fails, check the journal above. Common causes are a missing `target/web` build, a missing `dario-server` binary, or port 3041 being occupied:

```sh
ss -ltnp 'sport = :3041'
```

If the journal reports `target/web/index.html: No such file or directory`, the browser build is missing (or the unit's `WorkingDirectory` is wrong). The working directory must be `/home/danutz/Development/dario`. Building `dario-server` alone does not create the browser game. Run `make deploy` from the Mac to install both, or build the missing files on the Pi:

```sh
cd ~/Development/dario
rustup target add wasm32-unknown-unknown
sh scripts/build-web.sh &&
sudo systemctl reset-failed dario.service &&
sudo systemctl restart dario.service &&
systemctl status dario.service --no-pager
```

`reset-failed` clears the restart limit after repeated failures. The deployment installer also clears it before starting the service.

## Updates and port changes

Run `make deploy` again from the Mac for an update. The installer copies only the server binary, public browser files, and unit. It stops Dario only for the file replacement, after building finishes. Reload the browser once the update finishes. The manual build and upload steps above are also available.

To change the port, edit `Environment=DARIO_ADDR=...` in `/etc/systemd/system/dario.service`, then:

```sh
sudo systemctl daemon-reload
sudo systemctl restart dario.service
```

If using the installer afterward, keep `deploy/dario.service` and its port checks in `deploy/install.sh` consistent with your chosen port. Do not use `scripts/serve-web.sh` as `ExecStart`: that development script rebuilds at launch and binds to loopback.

To locate another service's unit and overrides:

```sh
systemctl cat solitaire.service
systemctl show solitaire.service -p FragmentPath -p DropInPaths
```

References: [systemd unit locations](https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html), [systemctl enable and start](https://www.freedesktop.org/software/systemd/man/latest/systemctl.html), [restart policy](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html), [tiny_http](https://docs.rs/tiny_http/0.12.0/tiny_http/).
