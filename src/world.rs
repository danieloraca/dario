use dario_progress::Progress;
use macroquad::prelude::{Rect, Vec2, vec2};

pub const STAGE_COUNT: usize = 3;
pub const WIDTH: f32 = 384.0;
pub const HEIGHT: f32 = 240.0;
pub const TILE: f32 = 16.0;
pub const STEP: f32 = 1.0 / 120.0;
const PLAYER_W: f32 = 12.0;
const PLAYER_H: f32 = 17.0;
const GRAVITY: f32 = 760.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Air,
    Ground,
    Brick,
    Question,
    Used,
    PipeLeft,
    PipeRight,
    Stone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Title,
    Playing,
    Paused,
    Dying,
    StageClear,
    GameOver,
    Won,
    LevelSelect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundEvent {
    Jump,
    Coin,
    Bump,
    Stomp,
    Hurt,
    Checkpoint,
    Clear,
}

#[derive(Clone, Copy, Default)]
pub struct Input {
    pub axis: f32,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub run: bool,
}

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub grounded: bool,
    pub facing: f32,
    pub invulnerable: f32,
    pub stride: f32,
    coyote: f32,
    jump_buffer: f32,
}

impl Player {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            grounded: false,
            facing: 1.0,
            invulnerable: 0.0,
            stride: 0.0,
            coyote: 0.0,
            jump_buffer: 0.0,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, PLAYER_W, PLAYER_H)
    }
}

pub struct Enemy {
    pub pos: Vec2,
    pub vel: Vec2,
    pub squished: Option<f32>,
}

impl Enemy {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: vec2(x, y),
            vel: vec2(-26.0, 0.0),
            squished: None,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, 14.0, 12.0)
    }
}

pub struct Coin {
    pub pos: Vec2,
    pub collected: bool,
}

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub gold: bool,
}

pub struct Level {
    pub name: &'static str,
    pub width: usize,
    pub tiles: Vec<Tile>,
    pub coins: Vec<Coin>,
    pub enemies: Vec<Enemy>,
    pub checkpoint: Vec2,
    pub goal: f32,
}

impl Level {
    pub fn new(stage: usize) -> Self {
        let (name, width, gaps): (&str, usize, &[(usize, usize)]) = match stage {
            0 => (
                "SUNNY SIDE UP",
                150,
                &[(29, 31), (62, 65), (99, 102), (121, 124)],
            ),
            1 => (
                "THE GOLDEN HOUR",
                164,
                &[(24, 27), (48, 51), (91, 94), (116, 120), (140, 143)],
            ),
            _ => (
                "ONE MORE SUNSET",
                178,
                &[(30, 34), (56, 59), (83, 87), (114, 118), (145, 149)],
            ),
        };
        let mut level = Self {
            name,
            width,
            tiles: vec![Tile::Air; width * 15],
            coins: vec![],
            enemies: vec![],
            checkpoint: vec2(74.0 * TILE, 12.0 * TILE - PLAYER_H),
            goal: (width - 10) as f32 * TILE,
        };
        for x in 0..width {
            if !gaps.iter().any(|&(start, end)| (start..end).contains(&x)) {
                for y in 12..15 {
                    level.set(x, y, Tile::Ground);
                }
            }
        }

        // Hand-placed beats: a safe introduction, low platforms, then wider jumps.
        let blocks: &[(usize, usize, usize)] = match stage {
            0 => &[
                (12, 9, 5),
                (22, 8, 3),
                (38, 9, 5),
                (53, 8, 4),
                (80, 9, 5),
                (109, 9, 4),
                (129, 9, 3),
            ],
            1 => &[
                (10, 9, 4),
                (19, 7, 3),
                (35, 9, 5),
                (58, 9, 4),
                (64, 7, 4),
                (81, 9, 5),
                (102, 8, 5),
                (126, 9, 5),
            ],
            _ => &[
                (12, 9, 4),
                (22, 7, 4),
                (40, 9, 5),
                (49, 7, 3),
                (66, 9, 4),
                (92, 9, 5),
                (104, 7, 3),
                (126, 9, 5),
                (136, 7, 4),
                (154, 9, 4),
            ],
        };
        for &(x, y, len) in blocks {
            for n in 0..len {
                level.set(
                    x + n,
                    y,
                    if n % 3 == 1 {
                        Tile::Question
                    } else {
                        Tile::Brick
                    },
                );
                if n % 2 == 0 {
                    level.add_coin(x + n, y - 2);
                }
            }
        }
        let pipes: &[(usize, usize)] = match stage {
            0 => &[(19, 2), (46, 2), (89, 3), (116, 2)],
            1 => &[(30, 2), (44, 3), (98, 2), (135, 3)],
            _ => &[(18, 2), (37, 3), (62, 2), (99, 3), (121, 2), (160, 2)],
        };
        for &(x, height) in pipes {
            for y in 12 - height..12 {
                level.set(x, y, Tile::PipeLeft);
                level.set(x + 1, y, Tile::PipeRight);
            }
        }
        for &(start, end) in gaps {
            for x in start.saturating_sub(1)..=end {
                level.add_coin(x, if x >= start && x < end { 8 } else { 9 });
            }
        }
        for x in [7, 8, 9, 70, 71, 72] {
            level.add_coin(x, 10);
        }
        let enemies: &[usize] = match stage {
            0 => &[23, 40, 56, 84, 96, 111, 131],
            1 => &[17, 39, 60, 84, 106, 128, 147],
            _ => &[25, 44, 68, 79, 94, 109, 130, 151, 164],
        };
        for n in 0..4 {
            for y in 12 - n..12 {
                level.set(width - 16 + n, y, Tile::Stone);
            }
        }
        for &x in enemies {
            let mut surface = 12;
            while level.tile(x as i32, surface - 1) != Tile::Air {
                surface -= 1;
            }
            level
                .enemies
                .push(Enemy::new(x as f32 * TILE, surface as f32 * TILE - 12.0));
        }
        level
    }

    fn add_coin(&mut self, x: usize, y: usize) {
        self.coins.push(Coin {
            pos: vec2(x as f32 * TILE + 4.0, y as f32 * TILE + 3.0),
            collected: false,
        });
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= 15 {
            return Tile::Air;
        }
        self.tiles[y as usize * self.width + x as usize]
    }

    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[y * self.width + x] = tile;
    }

    pub fn solids(&self, rect: Rect) -> Vec<(i32, i32, Rect)> {
        let mut result = Vec::new();
        for y in (rect.y / TILE).floor() as i32..=((rect.y + rect.h - 0.001) / TILE).floor() as i32
        {
            for x in
                (rect.x / TILE).floor() as i32..=((rect.x + rect.w - 0.001) / TILE).floor() as i32
            {
                if self.tile(x, y) != Tile::Air {
                    result.push((
                        x,
                        y,
                        Rect::new(x as f32 * TILE, y as f32 * TILE, TILE, TILE),
                    ));
                }
            }
        }
        result
    }
}

pub struct Game {
    pub phase: Phase,
    pub stage: usize,
    pub level: Level,
    pub player: Player,
    pub camera: f32,
    pub coins: u32,
    pub score: u32,
    pub lives: u8,
    pub checkpoint: bool,
    pub particles: Vec<Particle>,
    pub sounds: Vec<SoundEvent>,
    pub time: f32,
    pub phase_time: f32,
    pub banner_time: f32,
    pub bumped: Option<(i32, i32, f32)>,
    pub progress: Progress,
    pub selected_stage: usize,
    pub save_notice: Option<&'static str>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            phase: Phase::Title,
            stage: 0,
            level: Level::new(0),
            player: Player::new(vec2(56.0, 175.0)),
            camera: 0.0,
            coins: 0,
            score: 0,
            lives: 3,
            checkpoint: false,
            particles: vec![],
            sounds: vec![],
            time: 0.0,
            phase_time: 0.0,
            banner_time: 0.0,
            bumped: None,
            progress: Progress::default(),
            selected_stage: 0,
            save_notice: None,
        }
    }

    pub fn start(&mut self) {
        self.start_stage(self.progress.unlocked().min(STAGE_COUNT - 1));
    }

    pub fn start_stage(&mut self, stage: usize) {
        if stage >= STAGE_COUNT || stage > self.progress.unlocked() {
            return;
        }
        let progress = std::mem::take(&mut self.progress);
        let notice = self.save_notice;
        *self = Self::new();
        self.progress = progress;
        self.save_notice = notice;
        self.stage = stage;
        self.selected_stage = stage;
        self.level = Level::new(stage);
        self.phase = Phase::Playing;
        self.banner_time = 2.6;
    }

    pub fn select_levels(&mut self) {
        self.selected_stage = self.progress.unlocked().min(STAGE_COUNT - 1);
        self.phase = Phase::LevelSelect;
    }

    pub fn select_relative(&mut self, offset: isize) {
        self.selected_stage = self
            .selected_stage
            .saturating_add_signed(offset)
            .min(STAGE_COUNT - 1);
    }

    pub fn toggle_pause(&mut self) {
        self.phase = match self.phase {
            Phase::Playing => Phase::Paused,
            Phase::Paused => Phase::Playing,
            phase => phase,
        };
    }

    fn burst(&mut self, pos: Vec2, gold: bool) {
        for i in 0..7 {
            let angle = i as f32 * 2.399;
            self.particles.push(Particle {
                pos,
                vel: vec2(angle.cos() * 38.0, -25.0 - angle.sin().abs() * 65.0),
                life: 0.35 + i as f32 * 0.035,
                gold,
            });
        }
    }

    fn collect(&mut self, pos: Vec2) {
        self.coins += 1;
        self.score += 100;
        self.sounds.push(SoundEvent::Coin);
        self.burst(pos, true);
    }

    fn die(&mut self) {
        if self.phase != Phase::Playing {
            return;
        }
        self.phase = Phase::Dying;
        self.phase_time = 0.0;
        self.player.vel = vec2(0.0, -210.0);
        self.lives = self.lives.saturating_sub(1);
        self.sounds.push(SoundEvent::Hurt);
    }

    pub fn update(&mut self, input: Input, dt: f32) {
        if self.phase == Phase::Paused {
            return;
        }
        self.time += dt;
        if matches!(
            self.phase,
            Phase::Title | Phase::Won | Phase::GameOver | Phase::LevelSelect
        ) {
            return;
        }
        self.phase_time += dt;
        self.banner_time = (self.banner_time - dt).max(0.0);
        self.particles.retain_mut(|p| {
            p.life -= dt;
            p.vel.y += 240.0 * dt;
            p.pos += p.vel * dt;
            p.life > 0.0
        });
        if let Some((_, _, ref mut timer)) = self.bumped {
            *timer -= dt;
            if *timer <= 0.0 {
                self.bumped = None;
            }
        }
        if self.phase == Phase::Dying {
            self.player.vel.y += GRAVITY * dt;
            self.player.pos += self.player.vel * dt;
            if self.phase_time > 1.25 {
                if self.lives == 0 {
                    self.phase = Phase::GameOver;
                } else {
                    let spawn = if self.checkpoint {
                        self.level.checkpoint
                    } else {
                        vec2(56.0, 175.0)
                    };
                    self.player = Player::new(spawn);
                    self.player.invulnerable = 2.0;
                    self.camera = (spawn.x - 120.0).max(0.0);
                    self.phase = Phase::Playing;
                }
                self.phase_time = 0.0;
            }
            return;
        }
        if self.phase == Phase::StageClear {
            if self.phase_time > 2.6 {
                if self.stage + 1 == STAGE_COUNT {
                    self.phase = Phase::Won;
                } else {
                    self.stage += 1;
                    self.level = Level::new(self.stage);
                    self.player = Player::new(vec2(56.0, 175.0));
                    self.camera = 0.0;
                    self.checkpoint = false;
                    self.lives = 3;
                    self.particles.clear();
                    self.bumped = None;
                    self.phase = Phase::Playing;
                    self.banner_time = 2.6;
                }
                self.phase_time = 0.0;
            }
            return;
        }

        self.move_player(input, dt);
        self.move_enemies(dt);
        self.interact(input);
        let desired =
            (self.player.pos.x - 136.0).clamp(0.0, self.level.width as f32 * TILE - WIDTH);
        self.camera += (desired - self.camera) * (1.0 - (-8.0 * dt).exp());
    }

    fn move_player(&mut self, input: Input, dt: f32) {
        let p = &mut self.player;
        p.invulnerable = (p.invulnerable - dt).max(0.0);
        p.coyote = if p.grounded {
            0.10
        } else {
            (p.coyote - dt).max(0.0)
        };
        p.jump_buffer = if input.jump_pressed {
            0.12
        } else {
            (p.jump_buffer - dt).max(0.0)
        };
        let target_speed = input.axis * if input.run { 164.0 } else { 112.0 };
        let acceleration = if input.axis == 0.0 { 900.0 } else { 720.0 };
        p.vel.x += (target_speed - p.vel.x).clamp(-acceleration * dt, acceleration * dt);
        if input.axis != 0.0 {
            p.facing = input.axis.signum();
        }
        if p.jump_buffer > 0.0 && p.coyote > 0.0 {
            p.vel.y = -286.0;
            p.grounded = false;
            p.coyote = 0.0;
            p.jump_buffer = 0.0;
            self.sounds.push(SoundEvent::Jump);
        }
        if !input.jump_held && p.vel.y < -115.0 {
            p.vel.y = -115.0;
        }
        p.vel.y = (p.vel.y + GRAVITY * dt).min(430.0);
        p.stride += p.vel.x.abs() * dt;
        p.pos.x = (p.pos.x + p.vel.x * dt).max(0.0);
        for (_, _, tile) in self.level.solids(p.rect()) {
            if p.vel.x > 0.0 {
                p.pos.x = tile.x - PLAYER_W;
            } else if p.vel.x < 0.0 {
                p.pos.x = tile.x + TILE;
            }
            p.vel.x = 0.0;
        }
        p.pos.y += p.vel.y * dt;
        p.grounded = false;
        let mut hit = None;
        for (x, y, tile) in self.level.solids(p.rect()) {
            if p.vel.y > 0.0 {
                p.pos.y = tile.y - PLAYER_H;
                p.grounded = true;
            } else if p.vel.y < 0.0 {
                p.pos.y = tile.y + TILE;
                hit = Some((x, y));
            }
            p.vel.y = 0.0;
        }
        if let Some((x, y)) = hit {
            let kind = self.level.tile(x, y);
            if matches!(kind, Tile::Question | Tile::Brick | Tile::Used) {
                self.bumped = Some((x, y, 0.18));
                if kind == Tile::Question {
                    self.level.set(x as usize, y as usize, Tile::Used);
                    self.collect(vec2(x as f32 * TILE + 8.0, y as f32 * TILE - 8.0));
                } else {
                    self.sounds.push(SoundEvent::Bump);
                }
            }
        }
    }

    fn move_enemies(&mut self, dt: f32) {
        // Take the enemies temporarily so tile queries stay independent of entity mutation.
        let mut enemies = std::mem::take(&mut self.level.enemies);
        for enemy in &mut enemies {
            if let Some(timer) = &mut enemy.squished {
                *timer -= dt;
                continue;
            }
            if (enemy.pos.x - self.player.pos.x).abs() > WIDTH + 32.0 {
                continue;
            }
            enemy.pos.x += enemy.vel.x * dt;
            if let Some((_, _, wall)) = self.level.solids(enemy.rect()).first() {
                enemy.pos.x = if enemy.vel.x > 0.0 {
                    wall.x - 14.0
                } else {
                    wall.x + TILE
                };
                enemy.vel.x *= -1.0;
            }
            // Beetles patrol ledges instead of quietly falling into pits off screen.
            let toe = enemy.pos.x + if enemy.vel.x > 0.0 { 15.0 } else { -1.0 };
            if enemy.vel.y == 0.0
                && self.level.tile(
                    (toe / TILE).floor() as i32,
                    ((enemy.pos.y + 13.0) / TILE).floor() as i32,
                ) == Tile::Air
            {
                enemy.vel.x *= -1.0;
            }
            enemy.vel.y = (enemy.vel.y + GRAVITY * dt).min(400.0);
            enemy.pos.y += enemy.vel.y * dt;
            for (_, _, floor) in self.level.solids(enemy.rect()) {
                if enemy.vel.y > 0.0 {
                    enemy.pos.y = floor.y - 12.0;
                    enemy.vel.y = 0.0;
                }
            }
        }
        enemies.retain(|e| e.squished.is_none_or(|timer| timer > 0.0) && e.pos.y < 280.0);
        self.level.enemies = enemies;
    }

    fn interact(&mut self, input: Input) {
        let player_rect = self.player.rect();
        let mut collected = Vec::new();
        for coin in &mut self.level.coins {
            if !coin.collected
                && player_rect.overlaps(&Rect::new(coin.pos.x, coin.pos.y, 8.0, 10.0))
            {
                coin.collected = true;
                collected.push(coin.pos + vec2(4.0, 4.0));
            }
        }
        for pos in collected {
            self.collect(pos);
        }
        let mut stomped = Vec::new();
        let mut hurt = false;
        for enemy in &mut self.level.enemies {
            if enemy.squished.is_some() || !player_rect.overlaps(&enemy.rect()) {
                continue;
            }
            let feet_before = self.player.pos.y + PLAYER_H - self.player.vel.y * STEP;
            if self.player.vel.y > 0.0 && feet_before <= enemy.pos.y + 5.0 {
                enemy.squished = Some(0.3);
                self.player.pos.y = enemy.pos.y - PLAYER_H;
                self.player.vel.y = if input.jump_held { -260.0 } else { -185.0 };
                self.score += 200;
                stomped.push(enemy.pos + vec2(7.0, 5.0));
            } else if self.player.invulnerable <= 0.0 {
                hurt = true;
            }
        }
        for pos in stomped {
            self.sounds.push(SoundEvent::Stomp);
            self.burst(pos, false);
        }
        if hurt || self.player.pos.y > HEIGHT + 28.0 {
            self.die();
            return;
        }
        if !self.checkpoint && self.player.pos.x >= self.level.checkpoint.x {
            self.checkpoint = true;
            self.banner_time = 2.0;
            self.sounds.push(SoundEvent::Checkpoint);
            self.burst(self.level.checkpoint + vec2(0.0, -20.0), true);
        }
        if self.player.pos.x >= self.level.goal {
            self.score += 1000;
            self.progress.levels[self.stage].cleared = true;
            self.phase = Phase::StageClear;
            self.phase_time = 0.0;
            self.sounds.push(SoundEvent::Clear);
            self.burst(self.player.pos, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance(game: &mut Game, frames: usize, input: Input) {
        for i in 0..frames {
            game.update(
                Input {
                    jump_pressed: input.jump_pressed && i == 0,
                    ..input
                },
                STEP,
            );
        }
    }

    #[test]
    fn player_lands_on_ground_and_cannot_walk_through_a_pipe() {
        let mut game = Game::new();
        game.start();
        game.level.enemies.clear();
        advance(
            &mut game,
            360,
            Input {
                axis: 1.0,
                ..Input::default()
            },
        );
        assert!(game.player.grounded);
        assert_eq!(game.player.pos.y, 175.0);
        assert!((game.player.pos.x - (19.0 * TILE - PLAYER_W)).abs() < 0.01);
    }

    #[test]
    fn releasing_jump_makes_a_shorter_hop() {
        let peak = |held| {
            let mut game = Game::new();
            game.start();
            advance(&mut game, 2, Input::default());
            let mut highest = game.player.pos.y;
            for i in 0..80 {
                game.update(
                    Input {
                        jump_pressed: i == 0,
                        jump_held: held || i == 0,
                        ..Input::default()
                    },
                    STEP,
                );
                highest = highest.min(game.player.pos.y);
            }
            highest
        };
        assert!(peak(true) + 20.0 < peak(false));
    }

    #[test]
    fn coin_blocks_pay_once_and_stop_upward_motion() {
        let mut game = Game::new();
        game.start();
        game.level.coins.clear();
        game.player.pos = vec2(13.0 * TILE + 2.0, 175.0);
        advance(&mut game, 2, Input::default());
        advance(
            &mut game,
            100,
            Input {
                jump_pressed: true,
                jump_held: true,
                ..Input::default()
            },
        );
        assert_eq!(game.level.tile(13, 9), Tile::Used);
        assert_eq!(game.coins, 1);
        advance(
            &mut game,
            100,
            Input {
                jump_pressed: true,
                jump_held: true,
                ..Input::default()
            },
        );
        assert_eq!(game.coins, 1);
    }

    #[test]
    fn stomping_bounces_but_side_contact_costs_a_life() {
        let mut game = Game::new();
        game.start();
        game.level.enemies = vec![Enemy::new(100.0, 180.0)];
        game.player.pos = vec2(101.0, 163.0);
        game.player.vel.y = 120.0;
        game.update(Input::default(), STEP);
        assert!(game.level.enemies[0].squished.is_some());
        assert!(game.player.vel.y < 0.0);
        assert_eq!(game.score, 200);
        game.level.enemies = vec![Enemy::new(100.0, 180.0)];
        game.player.pos = vec2(99.0, 175.0);
        game.player.vel = Vec2::ZERO;
        game.update(Input::default(), STEP);
        assert_eq!(game.phase, Phase::Dying);
        assert_eq!(game.lives, 2);
    }

    #[test]
    fn checkpoint_respawn_preserves_collected_coins() {
        let mut game = Game::new();
        game.start();
        game.player.pos = game.level.checkpoint;
        game.update(Input::default(), STEP);
        assert!(game.checkpoint);
        game.coins = 12;
        game.player.pos.y = 300.0;
        advance(&mut game, 160, Input::default());
        assert_eq!(game.phase, Phase::Playing);
        assert_eq!(game.coins, 12);
        assert_eq!(game.lives, 2);
        assert!((game.player.pos.x - game.level.checkpoint.x).abs() < 1.0);
        assert!(game.player.invulnerable > 0.0);
    }

    #[test]
    fn campaign_progress_survives_retries_and_continues_at_the_next_level() {
        let mut game = Game::new();
        game.start();
        game.player.pos = vec2(game.level.goal, 130.0);
        game.update(Input::default(), STEP);
        assert!(game.progress.levels[0].cleared);
        let saved = Progress::decode(&game.progress.encode()).unwrap();
        let mut reopened = Game::new();
        reopened.progress = saved;
        reopened.start();
        assert_eq!(reopened.stage, 1);
        reopened.start_stage(2);
        assert_eq!(reopened.stage, 1, "locked levels cannot start");
        reopened.start_stage(0);
        assert_eq!(reopened.stage, 0);
        assert!(reopened.progress.levels[0].cleared);
    }

    #[test]
    fn all_three_exits_advance_to_victory() {
        let mut game = Game::new();
        game.start();
        for stage in 0..3 {
            assert_eq!(game.stage, stage);
            game.player.pos.x = game.level.goal;
            game.player.pos.y = 130.0;
            game.update(Input::default(), STEP);
            assert_eq!(game.phase, Phase::StageClear);
            advance(&mut game, 320, Input::default());
        }
        assert_eq!(game.phase, Phase::Won);
        assert_eq!(game.score, 3000);
    }

    #[test]
    fn pause_freezes_simulation_and_restart_resets_campaign() {
        let mut game = Game::new();
        game.start();
        game.toggle_pause();
        let pos = game.player.pos;
        advance(
            &mut game,
            120,
            Input {
                axis: 1.0,
                ..Input::default()
            },
        );
        assert_eq!(game.player.pos, pos);
        assert_eq!(game.time, 0.0);
        game.coins = 40;
        game.lives = 0;
        game.start();
        assert_eq!((game.coins, game.lives, game.stage), (0, 3, 0));
        assert_eq!(game.phase, Phase::Playing);
    }

    #[test]
    fn each_level_has_safe_spawn_checkpoint_and_exit() {
        for stage in 0..3 {
            let level = Level::new(stage);
            for x in [3, 4, 74, (level.goal / TILE) as i32] {
                assert_ne!(level.tile(x, 12), Tile::Air);
                assert_eq!(level.tile(x, 11), Tile::Air);
            }
            for enemy in &level.enemies {
                assert!(
                    level.solids(enemy.rect()).is_empty(),
                    "enemy spawned inside terrain in stage {stage}"
                );
            }
        }
    }

    #[test]
    fn running_jumps_clear_the_tallest_pipe_and_widest_gap() {
        // Exercise the actual level geometry at its two limiting obstacles.
        for (stage, start, end) in [
            (0, 89.0 * TILE - 58.0, 91.0 * TILE),
            (1, 116.0 * TILE - 20.0, 120.0 * TILE),
        ] {
            let mut game = Game::new();
            game.start();
            game.stage = stage;
            game.level = Level::new(stage);
            game.level.enemies.clear();
            game.player.pos = vec2(start, 175.0);
            advance(&mut game, 2, Input::default());
            game.player.vel.x = 164.0;
            advance(
                &mut game,
                110,
                Input {
                    axis: 1.0,
                    run: true,
                    jump_pressed: true,
                    jump_held: true,
                },
            );
            assert_eq!(game.phase, Phase::Playing);
            assert!(
                game.player.pos.x > end,
                "did not clear obstacle at {end}: {:?}",
                game.player.pos
            );
            assert!(game.player.grounded);
        }
    }
}
