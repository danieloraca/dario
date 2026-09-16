# Dario

A little Rust. A lot of jump.

A small, original 1980s-style platformer written in Rust. Run through three scrolling worlds, collect coins, bump coin blocks, hop over pipes and pits, stomp beetles, and reach the flag. Three lives, a checkpoint in each world, and one very determined little adventurer.

## Play

Install [Rust](https://www.rust-lang.org/tools/install), then:

```sh
cargo run --release
```

The first build downloads dependencies. Later runs work offline. No asset downloads, game ROMs, or external audio files are needed.

On macOS, you can also build a double-clickable app:

```sh
sh scripts/bundle-macos.sh
open target/Dario.app
```

This creates a locally signed app in `target/Dario.app`. It is intended for this Mac; it is not a notarized distribution build.

## Play in a browser

The browser version runs the same Rust game, compiled to WebAssembly, including all music and effects. Install the target once, then build the game and start its Rust web server (Python 3 is used only while assembling the browser build):

```sh
rustup target add wasm32-unknown-unknown
sh scripts/serve-web.sh
```

Open [Dario in your browser](http://127.0.0.1:8080) and press Enter. To use another port, run `sh scripts/serve-web.sh 8081`. Stop the server with Ctrl+C. Rerun the command after editing Rust or web files to rebuild; there is no hot reload.

A keyboard is required. The game fills the browser window and adapts as you resize it, revealing more scenery while keeping the pixel art in proportion. The controls below also work in the browser; **Q returns to the title screen** there. Audio unlocks on your first keypress or click. Leaving the tab pauses the game and silences its audio; press P to resume. Press **F** for fullscreen, which uses the browser's permission rules.

For a Raspberry Pi that serves the game on port 3041 and starts it automatically after reboot, use the [systemd deployment guide](deploy/README.md) and [Dario service file](deploy/dario.service). The service runs `target/release/dario-server` from `~/Development/dario`, with `DARIO_ADDR=0.0.0.0:3041`, matching the existing Solitaire service layout.

To deploy or redeploy from your Mac, run:

```sh
make deploy
```

This builds the browser game, uploads it to `danutz@192.168.0.25`, builds the server on the Pi, installs it into `~/Development/dario`, restarts only Dario, and checks the page and WASM response on port 3041. SSH and sudo prompt in your terminal as needed. The server build finishes before the running service is stopped. You can change the SSH destination with `make deploy PI_HOST=danutz@hostname`.

To run just the server once `target/web/` is built:

```sh
cargo build --release --locked -p dario-server
DARIO_ADDR=127.0.0.1:3041 ./target/release/dario-server
```

`dario-server` is a separate Rust package using [tiny_http](https://docs.rs/tiny_http/0.12.0/tiny_http/); it builds without the game's graphics or audio dependencies. It serves only the public game assets, supports GET and HEAD, and sends `.wasm` with `application/wasm`. Its default address is `127.0.0.1:3041`. Run it from the project directory; it checks that the complete browser build exists before listening. The `dario` binary still launches the desktop game.

For static hosting, run `sh scripts/build-web.sh` and publish the contents of `target/web/`. Serve `.wasm` as `application/wasm` over HTTP(S), rather than opening the HTML as a local file. The build assembles the graphics and audio JavaScript runtime from the versions of Miniquad and quad-snd in `Cargo.lock`, so it uses no CDN or third-party requests. This also avoids an unused plugin error in Macroquad 0.4.16's prebuilt JavaScript bundle. See [Macroquad's WebAssembly documentation](https://github.com/not-fl3/macroquad#wasm).

The game's [build script](build.rs) explicitly imports JavaScript-provided functions when linking WebAssembly, including quad-snd's audio functions. This avoids relying on older Rust linker defaults; the setting applies only to the game's `wasm32-unknown-unknown` build. Dario's own browser imports explicitly name the `env` module supplied by `web/game.js`.

| Key | Action |
| --- | --- |
| Arrow keys / A, D | Move |
| Space / Z / Up / W | Jump; hold for a higher jump |
| Shift / X | Run |
| Enter | Start / play again |
| Escape / P | Pause / resume |
| M | Mute / unmute all audio |
| F | Fullscreen / windowed |
| R | Restart the adventure |
| Q | Quit |

Hold jump when stomping a beetle for an extra bounce. Gold flags halfway through a level mark your checkpoint. Coins are worth 100 points, beetles 200, and each finish 1,000. Dying preserves collected coins and used blocks; restarting begins a fresh run.

## The little details

- Three handmade stages: **Sunny Side Up**, **The Golden Hour**, and **One More Sunset**.
- A 384 × 240 base canvas, an expanding browser viewport, nearest-neighbor scaling (integer scaling on desktop), parallax hills, animated coins, and a bold, homemade 7×7 arcade alphabet.
- Fixed 120 Hz physics, acceleration, variable jump height, a small grace period after leaving ledges, and buffered jumps just before landing.
- An original 16-bar chiptune with pulse-wave melody, triangle bass, arpeggios, and synthesized noise drums. Seven synthesized effects cover jumping, coins, bumps, stomps, damage, checkpoints, and finishes.
- All sprites, music, and sound effects are generated in Rust. No Nintendo artwork, recordings, or music are included.

Built with [Macroquad](https://macroquad.rs/); graphics and audio use its [official APIs](https://docs.rs/macroquad/0.4.16/macroquad/). The game builds for native desktop and WebAssembly. macOS is the verified native development platform. Linux builds require development packages for ALSA, X11, and OpenGL (on Debian/Ubuntu: `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev`).

## Development

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -- --smoke-test
```

The smoke test opens a real game window, simulates movement and a jump, and writes screenshots of the title, gameplay, pause, both later palettes, and victory to `target/smoke/`. It requires a graphical desktop and uses staged states to exercise the later screens; it is not an automated playthrough of every level.

```sh
cargo run -- --mute           # Start quietly
cargo run -- --export-music   # Save dario-theme.wav without opening a window
```

`src/world.rs` contains simulation and gameplay tests, `src/art.rs` draws the pixel art, `src/sound.rs` synthesizes PCM audio, and `src/main.rs` handles the window, keyboard, and fixed-step loop. `src/browser.rs` and `web/` provide the thin browser integration; the native build does not use them. `server/src/main.rs` provides the HTTP server and its request tests. The workspace defaults to the game, so `cargo run --release` continues to launch it.
# dario
