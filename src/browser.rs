//! The browser boundary. Simulation, artwork and audio stay shared with desktop.
use crate::world::{Game, Phase};

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn dario_status(phase: u32, world: u32, coins: u32, lives: u32, muted: u32);
    fn dario_take_pause_request() -> u32;
}

#[unsafe(no_mangle)]
pub extern "C" fn dario_web_crate_version() -> u32 {
    1
}

pub fn take_pause_request() -> bool {
    // These imports are supplied by web/game.js alongside the WASM module.
    unsafe { dario_take_pause_request() != 0 }
}

pub fn update_status(game: &Game, muted: bool) {
    let phase = match game.phase {
        Phase::Title => 0,
        Phase::Playing => 1,
        Phase::Paused => 2,
        Phase::Dying => 3,
        Phase::StageClear => 4,
        Phase::GameOver => 5,
        Phase::Won => 6,
    };
    unsafe {
        dario_status(
            phase,
            game.stage as u32 + 1,
            game.coins,
            game.lives as u32,
            u32::from(muted),
        )
    }
}
