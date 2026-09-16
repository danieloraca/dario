mod art;
#[cfg(target_arch = "wasm32")]
mod browser;
mod entities;
mod sound;
#[cfg(not(target_arch = "wasm32"))]
mod storage;
mod world;

use macroquad::prelude::*;
use world::{Game, HEIGHT, Input, Phase, STEP, WIDTH};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--export-music") {
        let path = "dario-theme.wav";
        match std::fs::write(path, sound::music_wav()) {
            Ok(()) => println!("Wrote the original Dario theme to {path}"),
            Err(error) => {
                eprintln!("Could not write {path}: {error}");
                std::process::exit(1);
            }
        }
        return;
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "DARIO - a tiny Rust platformer\n\nARROWS / A D  Move\nSPACE / Z / UP  Jump (hold to jump higher)\nSHIFT / X  Run\nENTER  Start / play again\nESC / P  Pause\nM  Toggle sound\nF  Fullscreen\nR  Restart\nQ  Quit\n\n--mute          Start with sound muted\n--export-music  Write dario-theme.wav without opening a window\n--smoke-test    Render title, play, pause and ending screenshots to target/smoke/"
        );
        return;
    }
    let smoke = args.iter().any(|arg| arg == "--smoke-test");
    let muted = smoke || args.iter().any(|arg| arg == "--mute");
    launch(muted, smoke);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    launch(false, false);
}

fn launch(muted: bool, smoke: bool) {
    macroquad::Window::from_config(
        Conf {
            window_title: "Dario — A little Rust. A lot of jump.".into(),
            window_width: 1152,
            window_height: 720,
            window_resizable: true,
            high_dpi: true,
            sample_count: 1,
            icon: Some(art::icon()),
            ..Default::default()
        },
        run(muted, smoke),
    );
}

async fn run(muted: bool, smoke: bool) {
    let mut game = Game::new();
    #[cfg(not(target_arch = "wasm32"))]
    let (mut storage, progress) = storage::Storage::load(smoke);
    #[cfg(target_arch = "wasm32")]
    let (progress, browser_save_enabled) = match browser::load_progress() {
        Ok(progress) => (progress, true),
        Err(_) => {
            game.save_notice = Some("SAVE UNAVAILABLE - PLAYING AS GUEST");
            (dario_progress::Progress::default(), false)
        }
    };
    game.progress = progress;
    #[cfg(target_arch = "wasm32")]
    let mut browser_saved = game.progress.clone();
    let mut audio = sound::Audio::new(muted || game.progress.muted).await;
    let mut target = render_target(WIDTH as u32, HEIGHT as u32);
    target.texture.set_filter(FilterMode::Nearest);
    let mut accumulator = 0.0;
    let mut pending_jump = false;
    #[cfg(not(target_arch = "wasm32"))]
    let mut fullscreen = false;
    #[cfg(target_arch = "wasm32")]
    let mut browser_status = None;
    let mut frame = 0;
    if smoke {
        std::fs::create_dir_all("target/smoke").expect("create smoke screenshot directory");
    }

    loop {
        if is_key_pressed(KeyCode::Q) {
            #[cfg(not(target_arch = "wasm32"))]
            break;
            #[cfg(target_arch = "wasm32")]
            {
                game.phase = Phase::Title;
                pending_jump = false;
                accumulator = 0.0;
            }
        }
        if is_key_pressed(KeyCode::M) {
            audio.toggle();
            game.progress.muted = audio.muted;
        }
        // Browser fullscreen must run directly in a trusted DOM input event.
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
            if game.phase == Phase::LevelSelect {
                game.phase = Phase::Title;
            } else {
                game.toggle_pause();
            }
            pending_jump = false;
        }
        #[cfg(target_arch = "wasm32")]
        if browser::take_pause_request() && game.phase == Phase::Playing {
            game.toggle_pause();
            pending_jump = false;
        }
        if is_key_pressed(KeyCode::L) {
            game.select_levels();
            pending_jump = false;
        }
        if game.phase == Phase::LevelSelect {
            for (key, offset) in [
                (KeyCode::Left, -1),
                (KeyCode::Right, 1),
                (KeyCode::Up, -4),
                (KeyCode::Down, 4),
            ] {
                if is_key_pressed(key) {
                    game.select_relative(offset);
                }
            }
            if is_key_pressed(KeyCode::Enter) {
                game.start_stage(game.selected_stage);
            }
        } else if is_key_pressed(KeyCode::R) {
            game.start_stage(game.stage);
            pending_jump = false;
            accumulator = 0.0;
        } else if is_key_pressed(KeyCode::Enter)
            && matches!(game.phase, Phase::Title | Phase::Won | Phase::GameOver)
        {
            if game.phase == Phase::GameOver {
                game.start_stage(game.stage);
            } else {
                game.start();
            }
            pending_jump = false;
            accumulator = 0.0;
        }
        let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
        let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
        pending_jump |= game.phase == Phase::Playing
            && [KeyCode::Space, KeyCode::Z, KeyCode::Up, KeyCode::W]
                .iter()
                .any(|&key| is_key_pressed(key));
        let mut input = Input {
            axis: i32::from(right) as f32 - i32::from(left) as f32,
            jump_pressed: false,
            jump_held: [KeyCode::Space, KeyCode::Z, KeyCode::Up, KeyCode::W]
                .iter()
                .any(|&key| is_key_down(key)),
            run: is_key_down(KeyCode::LeftShift)
                || is_key_down(KeyCode::RightShift)
                || is_key_down(KeyCode::X),
        };

        if smoke {
            match frame {
                2 => game.start(),
                102 => game.toggle_pause(),
                104 => {
                    game.toggle_pause();
                    game.stage = 1;
                    game.level = world::Level::new(1);
                }
                106 => {
                    game.stage = 2;
                    game.level = world::Level::new(2);
                    game.player.pos = vec2(game.level.goal - 70.0, 125.0);
                    game.camera = game.level.goal - 240.0;
                }
                108 => game.phase = Phase::Won,
                110 => break,
                _ => {}
            }
            input.axis = if (2..102).contains(&frame) { 1.0 } else { 0.0 };
            input.jump_held = (35..65).contains(&frame);
            pending_jump = frame == 35;
        }

        accumulator += if smoke {
            1.0 / 60.0
        } else {
            get_frame_time().min(0.1)
        };
        while accumulator >= STEP {
            input.jump_pressed = pending_jump;
            game.update(input, STEP);
            pending_jump = false;
            accumulator -= STEP;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            storage.sync(&game.progress, get_time());
            game.save_notice = storage.notice;
        }
        #[cfg(target_arch = "wasm32")]
        if browser_save_enabled && game.progress != browser_saved {
            browser::save_progress(&game.progress);
            browser_saved = game.progress.clone();
        }
        audio.sync_music(game.phase == Phase::Playing);
        for event in game.sounds.drain(..) {
            audio.play(event);
        }

        let screen = vec2(screen_width().max(1.0), screen_height().max(1.0));
        let fit = (screen.x / WIDTH).min(screen.y / HEIGHT);
        #[cfg(target_arch = "wasm32")]
        let view = vec2((screen.x / fit).ceil(), (screen.y / fit).ceil());
        #[cfg(not(target_arch = "wasm32"))]
        let view = vec2(WIDTH, HEIGHT);
        if target.texture.width() != view.x || target.texture.height() != view.y {
            target = render_target(view.x as u32, view.y as u32);
            target.texture.set_filter(FilterMode::Nearest);
        }
        // Extra browser space reveals more world; sprites keep their proportions.
        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, HEIGHT - view.y, view.x, view.y));
        camera.render_target = Some(target.clone());
        set_camera(&camera);
        let camera_x = (game.camera - (view.x - WIDTH) * 136.0 / WIDTH).clamp(
            0.0,
            (game.level.width as f32 * world::TILE - view.x).max(0.0),
        );
        art::draw_world(&game, view, camera_x);
        camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, view.x, view.y));
        camera.render_target = Some(target.clone());
        set_camera(&camera);
        art::draw_ui(&game, audio.muted, view);
        set_default_camera();
        clear_background(color_u8!(20, 32, 35, 255));
        #[cfg(target_arch = "wasm32")]
        let size = screen;
        #[cfg(not(target_arch = "wasm32"))]
        let size = view * if fit >= 1.0 { fit.floor() } else { fit };
        draw_texture_ex(
            &target.texture,
            ((screen_width() - size.x) / 2.0).floor(),
            ((screen_height() - size.y) / 2.0).floor(),
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                flip_y: true,
                ..Default::default()
            },
        );
        #[cfg(target_arch = "wasm32")]
        {
            let status = (game.phase, game.stage, game.coins, game.lives, audio.muted);
            if browser_status != Some(status) {
                browser::update_status(&game, audio.muted);
                browser_status = Some(status);
            }
        }
        if smoke {
            let screenshot = match frame {
                1 => Some("title"),
                101 => Some("playing"),
                103 => Some("paused"),
                105 => Some("golden-hour"),
                107 => Some("sunset"),
                109 => Some("victory"),
                _ => None,
            };
            if let Some(name) = screenshot {
                get_screen_data().export_png(&format!("target/smoke/{name}.png"));
                println!(
                    "Rendered {name}: phase={:?}, position={:?}, coins={}",
                    game.phase, game.player.pos, game.coins
                );
            }
        }
        frame += 1;
        next_frame().await;
    }
    audio.sync_music(false);
}
