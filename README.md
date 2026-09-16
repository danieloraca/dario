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
- A 384 × 240 canvas, nearest-neighbor integer scaling, parallax hills, animated coins, and a homemade bitmap alphabet.
- Fixed 120 Hz physics, acceleration, variable jump height, a small grace period after leaving ledges, and buffered jumps just before landing.
- An original 16-bar chiptune with pulse-wave melody, triangle bass, arpeggios, and synthesized noise drums. Seven synthesized effects cover jumping, coins, bumps, stomps, damage, checkpoints, and finishes.
- All sprites, music, and sound effects are generated in Rust. No Nintendo artwork, recordings, or music are included.

Built with [Macroquad](https://macroquad.rs/); graphics and audio use its [official APIs](https://docs.rs/macroquad/0.4.16/macroquad/). The game is a native desktop executable. macOS is the verified development platform. Linux builds require development packages for ALSA, X11, and OpenGL (on Debian/Ubuntu: `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev`).

## Development

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- --smoke-test
```

The smoke test opens a real game window, simulates movement and a jump, and writes screenshots of the title, gameplay, pause, both later palettes, and victory to `target/smoke/`. It requires a graphical desktop and uses staged states to exercise the later screens; it is not an automated playthrough of every level.

```sh
cargo run -- --mute           # Start quietly
cargo run -- --export-music   # Save dario-theme.wav without opening a window
```

`src/world.rs` contains simulation and gameplay tests, `src/art.rs` draws the pixel art, `src/sound.rs` synthesizes PCM audio, and `src/main.rs` handles the window, keyboard, and fixed-step loop.
# dario
