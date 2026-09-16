//! The browser boundary. Simulation, artwork and audio stay shared with desktop.
use crate::world::{Game, Phase};

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn dario_status(phase: u32, world: u32, coins: u32, lives: u32, muted: u32);
    fn dario_take_pause_request() -> u32;
    fn dario_load_progress(pointer: *mut u8, capacity: usize) -> usize;
    fn dario_save_progress(pointer: *const u8, length: usize);
}

#[unsafe(no_mangle)]
pub extern "C" fn dario_web_crate_version() -> u32 {
    1
}

pub fn take_pause_request() -> bool {
    // These imports are supplied by web/game.js alongside the WASM module.
    unsafe { dario_take_pause_request() != 0 }
}

pub fn load_progress() -> Result<dario_progress::Progress, String> {
    let mut bytes = vec![0; dario_progress::MAX_SAVE_BYTES];
    let length = unsafe { dario_load_progress(bytes.as_mut_ptr(), bytes.len()) };
    if length == 0 {
        return Ok(dario_progress::Progress::default());
    }
    if length > bytes.len() {
        return Err("Save is too large".into());
    }
    dario_progress::Progress::decode(&bytes[..length])
}

pub fn save_progress(progress: &dario_progress::Progress) {
    let bytes = progress.encode();
    // JavaScript copies the bytes synchronously before its asynchronous fetch.
    unsafe {
        dario_save_progress(bytes.as_ptr(), bytes.len());
    }
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
        Phase::LevelSelect => 7,
    };
    unsafe {
        dario_status(
            phase,
            if game.phase == Phase::LevelSelect {
                game.selected_stage as u32 + 1
            } else {
                game.stage as u32 + 1
            },
            game.coins,
            game.lives as u32,
            u32::from(muted),
        )
    }
}
