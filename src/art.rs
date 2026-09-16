use crate::entities::EnemyKind;
use macroquad::prelude::*;

use crate::world::{Game, HEIGHT, Mode, Phase, STAGE_COUNT, TILE, Tile, WIDTH, time_label};

const INK: Color = color_u8!(32, 53, 55, 255);
const CREAM: Color = color_u8!(255, 246, 211, 255);
const GOLD: Color = color_u8!(255, 202, 79, 255);
const ORANGE: Color = color_u8!(230, 122, 58, 255);
const RED: Color = color_u8!(194, 71, 52, 255);
const GLYPH_WIDTH: usize = 7;
const GLYPH_ADVANCE: usize = GLYPH_WIDTH + 1;

pub fn icon() -> macroquad::miniquad::conf::Icon {
    fn pixels<const N: usize>(side: usize) -> [u8; N] {
        let mut bytes = [0; N];
        for y in 0..side {
            for x in 0..side {
                let px = x * 16 / side;
                let py = y * 16 / side;
                let letter = (1..15).contains(&px)
                    && (1..15).contains(&py)
                    && glyph('D')[(py - 1) / 2] & (1 << (GLYPH_WIDTH - 1 - (px - 1) / 2)) != 0;
                let color = if letter { GOLD } else { INK };
                let offset = (y * side + x) * 4;
                bytes[offset..offset + 4].copy_from_slice(&[
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                    255,
                ]);
            }
        }
        bytes
    }
    macroquad::miniquad::conf::Icon {
        small: pixels(16),
        medium: pixels(32),
        big: pixels(64),
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x.round(), y.round(), w, h, color);
}

fn glyph(c: char) -> [u8; 7] {
    // Seven-pixel arcade capitals: broad strokes, clipped corners, fixed spacing.
    match c {
        'A' => [28, 54, 99, 99, 127, 99, 99],
        'B' => [126, 99, 99, 126, 99, 99, 126],
        'C' => [62, 99, 96, 96, 96, 99, 62],
        'D' => [124, 102, 99, 99, 99, 102, 124],
        'E' => [127, 96, 96, 124, 96, 96, 127],
        'F' => [127, 96, 96, 124, 96, 96, 96],
        'G' => [62, 99, 96, 111, 99, 99, 62],
        'H' => [99, 99, 99, 127, 99, 99, 99],
        'I' => [62, 28, 28, 28, 28, 28, 62],
        'J' => [31, 6, 6, 6, 102, 102, 60],
        'K' => [99, 102, 108, 120, 108, 102, 99],
        'L' => [96, 96, 96, 96, 96, 96, 127],
        'M' => [99, 119, 127, 107, 99, 99, 99],
        'N' => [99, 115, 123, 111, 103, 99, 99],
        'O' => [62, 99, 99, 99, 99, 99, 62],
        'P' => [126, 99, 99, 126, 96, 96, 96],
        'Q' => [62, 99, 99, 99, 107, 102, 61],
        'R' => [126, 99, 99, 126, 108, 102, 99],
        'S' => [62, 99, 96, 62, 3, 99, 62],
        'T' => [127, 28, 28, 28, 28, 28, 28],
        'U' => [99, 99, 99, 99, 99, 99, 62],
        'V' => [99, 99, 99, 99, 54, 28, 8],
        'W' => [99, 99, 99, 107, 127, 119, 99],
        'X' => [99, 54, 28, 8, 28, 54, 99],
        'Y' => [99, 99, 54, 28, 28, 28, 28],
        'Z' => [127, 3, 6, 12, 24, 48, 127],
        '0' => [62, 99, 103, 107, 115, 99, 62],
        '1' => [12, 28, 60, 12, 12, 12, 63],
        '2' => [62, 99, 3, 14, 56, 96, 127],
        '3' => [62, 99, 3, 30, 3, 99, 62],
        '4' => [6, 14, 30, 54, 127, 6, 6],
        '5' => [127, 96, 126, 3, 3, 99, 62],
        '6' => [30, 48, 96, 126, 99, 99, 62],
        '7' => [127, 99, 6, 12, 24, 24, 24],
        '8' => [62, 99, 99, 62, 99, 99, 62],
        '9' => [62, 99, 99, 63, 3, 6, 60],
        '!' => [28, 28, 28, 28, 28, 0, 28],
        '?' => [62, 99, 3, 14, 28, 0, 28],
        '-' => [0, 0, 0, 62, 0, 0, 0],
        '+' => [0, 28, 28, 127, 28, 28, 0],
        '/' => [3, 6, 12, 24, 48, 96, 64],
        ':' => [0, 28, 28, 0, 28, 28, 0],
        '.' => [0, 0, 0, 0, 0, 28, 28],
        '>' => [96, 48, 24, 12, 24, 48, 96],
        '<' => [3, 6, 12, 24, 12, 6, 3],
        _ => [0; 7],
    }
}

pub fn text(label: &str, x: f32, y: f32, size: f32, color: Color) {
    for (i, c) in label.chars().enumerate() {
        for (row, bits) in glyph(c.to_ascii_uppercase()).iter().enumerate() {
            for col in 0..GLYPH_WIDTH {
                if bits & (1 << (GLYPH_WIDTH - 1 - col)) != 0 {
                    rect(
                        x + (i * GLYPH_ADVANCE + col) as f32 * size,
                        y + row as f32 * size,
                        size,
                        size,
                        color,
                    );
                }
            }
        }
    }
}

fn text_width(label: &str, size: f32) -> f32 {
    (label.chars().count() as f32 * GLYPH_ADVANCE as f32 - 1.0).max(0.0) * size
}

fn centered(label: &str, y: f32, size: f32, color: Color, view_width: f32) {
    let width = text_width(label, size);
    text(label, (view_width - width) / 2.0, y, size, color);
}

fn cloud(x: f32, y: f32, size: f32, color: Color) {
    rect(x + 7.0 * size, y, 17.0 * size, 4.0 * size, color);
    rect(
        x + 3.0 * size,
        y + 4.0 * size,
        29.0 * size,
        5.0 * size,
        color,
    );
    rect(x, y + 9.0 * size, 37.0 * size, 6.0 * size, color);
    rect(
        x + 4.0 * size,
        y + 15.0 * size,
        29.0 * size,
        2.0 * size,
        color,
    );
}

fn hill(x: f32, base: f32, radius: f32, height: f32, color: Color) {
    for step in 0..(height as i32 / 4) {
        let rise = step as f32 * 4.0;
        let half_width = radius * (1.0 - (rise / height).powi(2)).sqrt();
        rect(
            x - half_width,
            base - rise,
            (half_width * 2.0 / 4.0).ceil() * 4.0,
            4.0,
            color,
        );
    }
}

fn background(game: &Game, view: Vec2, camera: f32) {
    let theme = game.stage / 4;
    let (sky, horizon, far, near, sun) = match theme {
        0 => (
            color_u8!(112, 190, 186, 255),
            color_u8!(194, 223, 177, 255),
            color_u8!(129, 180, 132, 255),
            color_u8!(73, 135, 98, 255),
            color_u8!(255, 225, 148, 255),
        ),
        1 => (
            color_u8!(25, 34, 61, 255),
            color_u8!(59, 80, 119, 255),
            color_u8!(45, 56, 89, 255),
            color_u8!(67, 106, 146, 255),
            color_u8!(171, 237, 224, 255),
        ),
        2 => (
            color_u8!(73, 160, 200, 255),
            color_u8!(201, 233, 223, 255),
            color_u8!(166, 212, 213, 255),
            color_u8!(217, 239, 226, 255),
            color_u8!(255, 240, 188, 255),
        ),
        _ => (
            color_u8!(49, 31, 46, 255),
            color_u8!(138, 66, 58, 255),
            color_u8!(73, 41, 52, 255),
            color_u8!(108, 51, 50, 255),
            color_u8!(255, 171, 96, 255),
        ),
    };
    clear_background(sky);
    let extra_height = view.y - HEIGHT;
    let rows = 24 + (extra_height / 7.0).ceil() as i32;
    for row in 0..rows {
        let blend = row as f32 / (rows - 1) as f32;
        let color = Color::new(
            sky.r + (horizon.r - sky.r) * blend,
            sky.g + (horizon.g - sky.g) * blend,
            sky.b + (horizon.b - sky.b) * blend,
            1.0,
        );
        rect(
            0.0,
            32.0 - extra_height + row as f32 * 7.0,
            view.x,
            7.0,
            color,
        );
    }
    if theme == 1 {
        for i in -1..(view.x / 60.0).ceil() as i32 + 2 {
            let x = i as f32 * 60.0 - (camera * 0.3).rem_euclid(60.0);
            let height = 36 + (i * 17).rem_euclid(50);
            for row in 0..height / 4 {
                let width = ((height / 4 - row) / 2 + 2) as f32;
                rect(
                    x + 25.0 - width,
                    192.0 - row as f32 * 4.0,
                    width * 2.0,
                    4.0,
                    near,
                );
                rect(x + 25.0, 192.0 - row as f32 * 4.0, width, 4.0, far);
            }
            rect(
                x,
                0.0 - extra_height,
                28.0,
                41.0 + (i * 11).rem_euclid(29) as f32 + extra_height,
                far,
            );
            rect(
                x + 24.0,
                150.0 - (i * 13).rem_euclid(39) as f32,
                2.0,
                3.0,
                sun,
            );
        }
        return;
    }
    if theme == 3 {
        for i in -1..(view.x / 72.0).ceil() as i32 + 2 {
            let x = i as f32 * 72.0 - (camera * 0.25).rem_euclid(72.0);
            let top = 66.0 + (i * 17).rem_euclid(55) as f32;
            rect(x, top, 42.0, 150.0, far);
            rect(x - 3.0, top, 48.0, 5.0, near);
            for window in 0..4 {
                rect(x + 8.0, top + 14.0 + window as f32 * 18.0, 5.0, 7.0, ORANGE);
                rect(x + 27.0, top + 14.0 + window as f32 * 18.0, 5.0, 7.0, RED);
            }
            let ember = (game.time * 12.0 + i as f32 * 31.0).rem_euclid(150.0);
            rect(x + 50.0, 190.0 - ember, 2.0, 2.0, GOLD);
        }
        return;
    }
    let sun_x = view.x - 75.0 - camera * 0.025;
    for y in -19_i32..20 {
        let half = (20.0_f32.powi(2) - (y as f32).powi(2)).sqrt();
        rect(
            sun_x - half,
            69.0 - extra_height * 0.65 + y as f32,
            (half * 2.0).floor(),
            1.0,
            sun,
        );
    }
    let cloud_span = (view.x + 160.0).max(760.0);
    for i in -1_i32..(cloud_span / 113.0).ceil() as i32 {
        let x = (i as f32 * 113.0 - camera * 0.12 - game.time * 1.2).rem_euclid(cloud_span) - 80.0;
        cloud(
            x,
            46.0 + (i * 37).rem_euclid(43) as f32 - extra_height * i.rem_euclid(3) as f32 / 3.0,
            if i % 2 == 0 { 1.0 } else { 0.75 },
            Color::new(CREAM.r, CREAM.g, CREAM.b, 0.64),
        );
    }
    for i in -1_i32..(view.x / 155.0).ceil() as i32 + 1 {
        let x = i as f32 * 155.0 - (camera * 0.22).rem_euclid(155.0);
        hill(
            x + 55.0,
            194.0,
            94.0,
            60.0 + (i * 13).rem_euclid(25) as f32,
            far,
        );
    }
    for i in -1_i32..(view.x / 105.0).ceil() as i32 + 1 {
        let x = i as f32 * 105.0 - (camera * 0.43).rem_euclid(105.0);
        hill(
            x + 30.0,
            197.0,
            68.0,
            28.0 + (i * 19).rem_euclid(20) as f32,
            near,
        );
        rect(
            x + 19.0,
            169.0,
            2.0,
            5.0,
            Color::new(near.r * 0.83, near.g * 0.83, near.b * 0.83, 1.0),
        );
        rect(
            x + 28.0,
            169.0,
            2.0,
            5.0,
            Color::new(near.r * 0.83, near.g * 0.83, near.b * 0.83, 1.0),
        );
    }
    if theme == 2 {
        for i in 0..(view.x / 24.0).ceil() as i32 {
            let x = (i * 73 + 27) % view.x as i32;
            let y = 36.0 + (i * 31) as f32 % (70.0 + extra_height) - extra_height;
            rect(x as f32, y, 1.0, 2.0, CREAM);
        }
    }
}

fn tile(game: &Game, kind: Tile, tx: i32, ty: i32, x: f32, y: f32) {
    let (earth, highlight, top, grass) = match game.stage / 4 {
        1 => (
            color_u8!(54, 66, 103, 255),
            color_u8!(74, 88, 134, 255),
            color_u8!(158, 205, 233, 255),
            color_u8!(96, 137, 172, 255),
        ),
        2 => (
            color_u8!(81, 120, 141, 255),
            color_u8!(114, 157, 169, 255),
            CREAM,
            color_u8!(187, 222, 220, 255),
        ),
        3 => (
            color_u8!(78, 49, 51, 255),
            color_u8!(101, 61, 57, 255),
            color_u8!(215, 103, 54, 255),
            color_u8!(127, 64, 52, 255),
        ),
        _ => (
            color_u8!(163, 96, 62, 255),
            color_u8!(180, 112, 70, 255),
            color_u8!(205, 217, 113, 255),
            color_u8!(102, 151, 73, 255),
        ),
    };
    match kind {
        Tile::Air => {}
        Tile::Ground => {
            rect(x, y, 16.0, 16.0, earth);
            rect(x + 1.0, y + 1.0, 14.0, 14.0, highlight);
            if game.level.tile(tx, ty - 1) == Tile::Air {
                rect(x, y, 16.0, 3.0, top);
                rect(x, y + 3.0, 16.0, 4.0, grass);
                rect(x + 2.0, y + 6.0, 3.0, 2.0, grass);
                rect(x + 11.0, y + 6.0, 2.0, 3.0, grass);
            }
            let shift = (tx * 7 + ty * 3).rem_euclid(9) as f32;
            rect(
                x + 2.0 + shift,
                y + 10.0,
                3.0,
                2.0,
                color_u8!(213, 144, 85, 255),
            );
            rect(
                x + 10.0 - shift / 2.0,
                y + 13.0,
                2.0,
                1.0,
                color_u8!(128, 81, 58, 255),
            );
        }
        Tile::Brick => {
            rect(x, y, 16.0, 16.0, color_u8!(108, 67, 47, 255));
            rect(x + 1.0, y + 1.0, 14.0, 6.0, ORANGE);
            rect(x + 1.0, y + 9.0, 14.0, 6.0, color_u8!(202, 104, 56, 255));
            rect(x + 1.0, y + 1.0, 14.0, 1.0, color_u8!(248, 176, 92, 255));
            rect(x + 7.0, y, 1.0, 8.0, color_u8!(108, 67, 47, 255));
            rect(x + 3.0, y + 8.0, 1.0, 8.0, color_u8!(108, 67, 47, 255));
            rect(x + 12.0, y + 8.0, 1.0, 8.0, color_u8!(108, 67, 47, 255));
        }
        Tile::Question | Tile::Used => {
            let used = kind == Tile::Used;
            rect(
                x,
                y,
                16.0,
                16.0,
                if used {
                    color_u8!(109, 83, 55, 255)
                } else {
                    color_u8!(159, 98, 38, 255)
                },
            );
            rect(
                x + 1.0,
                y + 1.0,
                14.0,
                13.0,
                if used {
                    color_u8!(157, 125, 80, 255)
                } else {
                    GOLD
                },
            );
            rect(
                x + 2.0,
                y + 1.0,
                12.0,
                1.0,
                if used {
                    color_u8!(187, 151, 99, 255)
                } else {
                    CREAM
                },
            );
            for (dx, dy) in [(2.0, 3.0), (13.0, 3.0), (2.0, 12.0), (13.0, 12.0)] {
                rect(x + dx, y + dy, 1.0, 1.0, color_u8!(159, 98, 38, 255));
            }
            if !used {
                text("?", x + 5.0, y + 5.0, 1.0, ORANGE);
                text("?", x + 4.0, y + 4.0, 1.0, CREAM);
            }
        }
        Tile::PipeLeft | Tile::PipeRight => {
            let left = kind == Tile::PipeLeft;
            let origin = x - if left { 0.0 } else { 16.0 };
            if left {
                rect(origin + 2.0, y, 28.0, 16.0, color_u8!(41, 88, 70, 255));
                rect(origin + 4.0, y, 24.0, 16.0, color_u8!(66, 130, 84, 255));
                rect(origin + 6.0, y, 6.0, 16.0, color_u8!(141, 183, 99, 255));
                rect(origin + 7.0, y, 2.0, 16.0, color_u8!(196, 214, 129, 255));
                rect(origin + 23.0, y, 4.0, 16.0, color_u8!(49, 108, 77, 255));
                if game.level.tile(tx, ty - 1) != Tile::PipeLeft {
                    rect(origin, y, 32.0, 7.0, color_u8!(41, 88, 70, 255));
                    rect(
                        origin + 1.0,
                        y + 1.0,
                        30.0,
                        4.0,
                        color_u8!(111, 164, 94, 255),
                    );
                    rect(
                        origin + 2.0,
                        y + 1.0,
                        27.0,
                        1.0,
                        color_u8!(207, 223, 143, 255),
                    );
                    rect(
                        origin + 3.0,
                        y + 2.0,
                        7.0,
                        3.0,
                        color_u8!(169, 199, 119, 255),
                    );
                }
            }
        }
        Tile::Stone => {
            rect(x, y, 16.0, 16.0, color_u8!(110, 107, 81, 255));
            rect(x + 1.0, y + 1.0, 13.0, 13.0, color_u8!(181, 174, 125, 255));
            rect(x + 2.0, y + 1.0, 12.0, 2.0, color_u8!(219, 211, 155, 255));
            rect(x + 3.0, y + 5.0, 8.0, 6.0, color_u8!(161, 157, 110, 255));
        }
    }
}

fn coin(x: f32, y: f32, time: f32) {
    let width = [7.0, 5.0, 2.0, 5.0][(time * 7.0) as usize % 4];
    let x = x + (8.0 - width) / 2.0;
    rect(x, y + 2.0, width, 7.0, color_u8!(174, 115, 43, 255));
    rect(x + 1.0, y, (width - 2.0).max(1.0), 11.0, GOLD);
    rect(x, y + 2.0, width, 6.0, GOLD);
    rect(x + 1.0, y + 2.0, 1.0, 5.0, CREAM);
    if width > 4.0 {
        rect(x + width - 2.0, y + 3.0, 1.0, 5.0, ORANGE);
    }
}

fn challenge_coin(x: f32, y: f32, earned: bool) {
    let color = if earned {
        color_u8!(92, 222, 210, 255)
    } else {
        color_u8!(84, 110, 110, 255)
    };
    for (dy, width) in [2.0, 4.0, 6.0, 8.0, 8.0, 6.0, 4.0, 2.0].iter().enumerate() {
        rect(x + (8.0 - width) / 2.0, y + dy as f32, *width, 1.0, color);
    }
    if earned {
        rect(x + 3.0, y + 2.0, 2.0, 2.0, CREAM);
    }
}

fn sprite(rows: &[&str], x: f32, y: f32, facing: f32) {
    for (dy, row) in rows.iter().enumerate() {
        for (dx, pixel) in row.chars().enumerate() {
            let color = match pixel {
                'o' => INK,
                'r' => RED,
                'h' => ORANGE,
                's' => color_u8!(239, 174, 115, 255),
                'c' => CREAM,
                'b' => color_u8!(45, 98, 110, 255),
                't' => color_u8!(89, 152, 157, 255),
                'y' => GOLD,
                _ => continue,
            };
            let px = if facing >= 0.0 {
                dx
            } else {
                row.len() - 1 - dx
            };
            rect(x + px as f32, y + dy as f32, 1.0, 1.0, color);
        }
    }
}

fn player(game: &Game, camera: f32) {
    let p = &game.player;
    if p.invulnerable > 0.0 && (game.time * 12.0) as i32 % 2 == 0 {
        return;
    }
    let x = (p.pos.x - camera - 2.0).round();
    let y = (p.pos.y - 1.0).round();
    sprite(
        &[
            ".....oooooo.....",
            "....orhhhhrro...",
            "....orrryrrro...",
            "...orrrrrrrrro..",
            "...ooosssosoo...",
            "....oscccosso...",
            "....ossssoosso..",
            ".....ossssso....",
            "....orrrrro.....",
            "...orbrrbrrso...",
            "..ossbttbrosso..",
            "..osobyybooso...",
            ".....bbbbbo.....",
            ".....btbbbo.....",
        ],
        x,
        y,
        p.facing,
    );
    let airborne = game.phase != Phase::Title && (!p.grounded || game.phase == Phase::Dying);
    let walking = p.vel.x.abs() > 5.0 && (p.stride / 9.0) as i32 % 2 == 0;
    let legs: &[&str] = if airborne {
        &["....obboobbo....", "...obbo..oboo...", "...ooo....ooo..."]
    } else if walking {
        &["....obbo.obbo...", "...obbo...obbo..", "...oooo...oooo.."]
    } else {
        &[".....obobbo.....", ".....obobbo.....", "....ooo.oooo...."]
    };
    sprite(legs, x, y + 14.0, p.facing);
}

fn scenery(game: &Game, width: f32, camera: f32) {
    let start = (camera / TILE) as i32 - 1;
    let end = start + (width / TILE).ceil() as i32 + 3;
    for tx in start..end {
        if game.level.tile(tx, 12) == Tile::Ground {
            let x = tx as f32 * TILE - camera;
            if tx.rem_euclid(11) == 5 {
                rect(x + 8.0, 184.0, 1.0, 8.0, color_u8!(64, 111, 69, 255));
                rect(x + 5.0, 184.0, 7.0, 2.0, CREAM);
                rect(x + 7.0, 182.0, 3.0, 6.0, CREAM);
                rect(x + 7.0, 184.0, 3.0, 2.0, GOLD);
            }
            if tx.rem_euclid(7) == 2 {
                rect(x + 4.0, 189.0, 1.0, 3.0, color_u8!(59, 117, 72, 255));
                rect(x + 6.0, 187.0, 1.0, 5.0, color_u8!(59, 117, 72, 255));
                rect(x + 8.0, 189.0, 1.0, 3.0, color_u8!(59, 117, 72, 255));
            }
        }
    }
    // A small trail marker introduces the direction of travel.
    if camera < 160.0 {
        let x = 101.0 - camera;
        rect(x + 8.0, 174.0, 3.0, 18.0, color_u8!(120, 83, 56, 255));
        rect(x, 168.0, 24.0, 12.0, color_u8!(120, 83, 56, 255));
        rect(x + 1.0, 169.0, 22.0, 9.0, color_u8!(226, 178, 110, 255));
        text(">", x + 9.0, 170.0, 1.0, INK);
    }
    if game.mode != Mode::TimeTrial {
        let cp = game.level.checkpoint.x - camera;
        rect(cp + 3.0, 155.0, 2.0, 37.0, INK);
        rect(
            cp + 5.0,
            156.0,
            15.0,
            11.0,
            if game.checkpoint { GOLD } else { CREAM },
        );
        text(
            "+",
            cp + 8.0,
            158.0,
            1.0,
            if game.checkpoint {
                RED
            } else {
                color_u8!(105, 150, 113, 255)
            },
        );
    }
    let flag = game.level.goal - camera;
    rect(flag + 3.0, 91.0, 3.0, 101.0, INK);
    rect(flag + 3.0, 92.0, 1.0, 100.0, CREAM);
    rect(flag + 1.0, 87.0, 7.0, 6.0, GOLD);
    let flutter = (game.time * 6.0).sin().round();
    rect(flag + 6.0, 96.0, 22.0, 13.0, RED);
    rect(flag + 19.0, 96.0 + flutter, 13.0, 13.0, RED);
    sprite(
        &["..c..", ".ccc.", "ccccc", ".ccc.", "..c.."],
        flag + 13.0,
        100.0,
        1.0,
    );
    let house = flag + 57.0;
    rect(house, 151.0, 51.0, 41.0, color_u8!(228, 197, 140, 255));
    for i in 0..7 {
        rect(
            house - 5.0 + i as f32 * 4.0,
            150.0 - i as f32 * 3.0,
            61.0 - i as f32 * 8.0,
            4.0,
            RED,
        );
    }
    rect(house + 19.0, 168.0, 14.0, 24.0, INK);
    rect(house + 20.0, 169.0, 12.0, 22.0, color_u8!(85, 108, 88, 255));
    rect(house + 29.0, 181.0, 2.0, 2.0, GOLD);
    for dx in [5.0, 38.0] {
        rect(house + dx, 160.0, 9.0, 11.0, INK);
        rect(house + dx + 1.0, 161.0, 7.0, 9.0, GOLD);
        rect(house + dx + 4.0, 161.0, 1.0, 9.0, INK);
        rect(house + dx + 1.0, 165.0, 7.0, 1.0, INK);
    }
}

fn heart(x: f32, y: f32, full: bool) {
    let color = if full {
        color_u8!(246, 142, 111, 255)
    } else {
        color_u8!(79, 98, 89, 255)
    };
    for (row, bits) in [54_u8, 127, 127, 127, 62, 28, 8].iter().enumerate() {
        for col in 0..7 {
            if bits & (1 << (6 - col)) != 0 {
                rect(x + col as f32, y + row as f32, 1.0, 1.0, color);
            }
        }
    }
}

fn hud(game: &Game, muted: bool, view: Vec2) {
    let extra = view.x - WIDTH;
    rect(0.0, 0.0, view.x, 31.0, INK);
    rect(0.0, 30.0, view.x, 1.0, color_u8!(66, 91, 78, 255));
    text(
        match game.mode {
            Mode::Campaign => "DARIO",
            Mode::TimeTrial => "TRIAL",
            Mode::Arcade => "ARCADE",
        },
        12.0,
        7.0,
        1.0,
        CREAM,
    );
    for i in 0..3 {
        heart(12.0 + i as f32 * 11.0, 18.0, i < game.lives);
    }
    let dim = color_u8!(156, 184, 160, 255);
    text("TIME", 64.0 + extra * 0.2, 6.0, 1.0, dim);
    text(
        &time_label(game.elapsed_ms()),
        64.0 + extra * 0.2,
        18.0,
        1.0,
        CREAM,
    );
    coin(129.0 + extra * 0.35, 11.0, 0.0);
    text(
        &format!("{:02}", game.coins),
        140.0 + extra * 0.35,
        13.0,
        1.0,
        GOLD,
    );
    text("SCORE", 179.0 + extra * 0.5, 6.0, 1.0, dim);
    text(
        &format!("{:06}", game.score),
        179.0 + extra * 0.5,
        18.0,
        1.0,
        CREAM,
    );
    text("WORLD", 247.0 + extra * 0.7, 6.0, 1.0, dim);
    text(
        &crate::levels::world_label(game.stage),
        255.0 + extra * 0.7,
        18.0,
        1.0,
        CREAM,
    );
    text(
        if muted { "M / OFF" } else { "M / ON" },
        303.0 + extra * 0.85,
        6.0,
        1.0,
        CREAM,
    );
    for i in 0..3 {
        challenge_coin(
            303.0 + extra * 0.85 + i as f32 * 12.0,
            18.0,
            i < game.challenge_count(),
        );
    }
    text("II", view.x - 18.0, 15.0, 1.0, CREAM);
    rect(0.0, view.y - 8.0, view.x, 8.0, INK);
    let progress = (game.player.pos.x / game.level.goal).clamp(0.0, 1.0);
    rect(
        12.0,
        view.y - 5.0,
        view.x - 24.0,
        2.0,
        color_u8!(76, 101, 81, 255),
    );
    rect(
        12.0,
        view.y - 5.0,
        ((view.x - 24.0) * progress).max(2.0),
        2.0,
        GOLD,
    );
    if game.banner_time > 0.0 && game.phase == Phase::Playing {
        let label = if game.checkpoint {
            "CHECKPOINT!"
        } else {
            game.level.name
        };
        let width = text_width(label, 1.0) + 20.0;
        rect((view.x - width) / 2.0, 43.0, width, 19.0, INK);
        centered(label, 49.0, 1.0, CREAM, view.x);
    }
}

fn title(game: &Game, muted: bool, view: Vec2) {
    let dx = (view.x - WIDTH) / 2.0;
    let dy = (view.y - HEIGHT) / 2.0;
    text("DARIO / 01", 13.0, 12.0, 1.0, INK);
    let credit = "A RUST ORIGINAL";
    text(
        credit,
        view.x - 13.0 - text_width(credit, 1.0),
        12.0,
        1.0,
        INK,
    );
    centered(
        "SMALL GAME. BIG LITTLE ADVENTURE.",
        38.0 + dy,
        1.0,
        INK,
        view.x,
    );
    // Chunky, offset lettering is drawn with the same hand-made bitmap alphabet.
    let x = (view.x - text_width("DARIO", 7.0)) / 2.0;
    for (sx, sy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 7.0), (3.0, 5.0)] {
        text("DARIO", x + sx, 57.0 + dy + sy, 7.0, INK);
    }
    text("DARIO", x, 61.0 + dy, 7.0, RED);
    text("DARIO", x, 57.0 + dy, 7.0, GOLD);
    centered(
        &format!("ARCADE BEST {:06}", game.progress.arcade_best),
        119.0 + dy,
        1.0,
        INK,
        view.x,
    );
    rect(
        94.0 + dx,
        137.0 + dy,
        200.0,
        23.0,
        color_u8!(43, 80, 67, 255),
    );
    rect(92.0 + dx, 134.0 + dy, 200.0, 23.0, INK);
    rect(
        93.0 + dx,
        135.0 + dy,
        198.0,
        1.0,
        color_u8!(101, 136, 103, 255),
    );
    centered(
        if game.progress.levels[0].cleared {
            "ENTER TO CONTINUE"
        } else {
            "PRESS ENTER TO PLAY"
        },
        142.0 + dy,
        1.0,
        CREAM,
        view.x,
    );
    rect(44.0 + dx, 165.0 + dy, 296.0, 32.0, INK);
    centered("L LEVELS    T TIME TRIAL", 173.0 + dy, 1.0, CREAM, view.x);
    centered(
        "C ARCADE - 3 LIVES / 16 LEVELS",
        187.0 + dy,
        1.0,
        CREAM,
        view.x,
    );
    if (game.time * 2.0) as i32 % 2 == 0 {
        text(">", 102.0 + dx, 142.0 + dy, 1.0, GOLD);
    }
    rect(0.0, view.y - 38.0, view.x, 38.0, INK);
    rect(0.0, view.y - 39.0, view.x, 1.0, color_u8!(94, 129, 94, 255));
    centered(
        "ARROWS / A D  MOVE     SPACE / Z  JUMP",
        view.y - 30.0,
        1.0,
        CREAM,
        view.x,
    );
    centered(
        if muted {
            "SHIFT  RUN    M  SOUND OFF    F  FULLSCREEN"
        } else {
            "SHIFT  RUN    M  SOUND ON     F  FULLSCREEN"
        },
        view.y - 16.0,
        1.0,
        color_u8!(166, 189, 156, 255),
        view.x,
    );
}

fn panel(title: &str, subtitle: &str, action: &str, game: &Game, view: Vec2) {
    let dx = (view.x - WIDTH) / 2.0;
    let dy = (view.y - HEIGHT) / 2.0;
    rect(
        0.0,
        31.0,
        view.x,
        view.y - 31.0,
        Color::new(0.06, 0.12, 0.13, 0.62),
    );
    rect(
        47.0 + dx,
        66.0 + dy,
        294.0,
        120.0,
        Color::new(0.04, 0.08, 0.08, 0.4),
    );
    rect(44.0 + dx, 62.0 + dy, 294.0, 120.0, INK);
    rect(
        45.0 + dx,
        63.0 + dy,
        292.0,
        1.0,
        color_u8!(136, 161, 124, 255),
    );
    centered(title, 82.0 + dy, 2.0, GOLD, view.x);
    centered(subtitle, 110.0 + dy, 1.0, CREAM, view.x);
    if matches!(game.phase, Phase::Won | Phase::GameOver) {
        centered(
            &format!("{:02} COINS    {:06} POINTS", game.coins, game.score),
            130.0 + dy,
            1.0,
            color_u8!(166, 189, 156, 255),
            view.x,
        );
    }
    if game.phase == Phase::StageClear
        && let Some(result) = &game.finish
    {
        centered(
            &format!(
                "TIME {}   SCORE {}",
                time_label(result.millis),
                result.score
            ),
            124.0 + dy,
            1.0,
            CREAM,
            view.x,
        );
        centered(
            &format!(
                "CLEAR +   SPEED {}   GEMS {}",
                if result.speed { "+" } else { "-" },
                if result.treasure { "+" } else { "-" }
            ),
            139.0 + dy,
            1.0,
            GOLD,
            view.x,
        );
    }
    centered(action, 157.0 + dy, 1.0, CREAM, view.x);
}

pub fn draw_world(game: &Game, view: Vec2, camera: f32) {
    background(game, view, camera);
    scenery(game, view.x, camera);
    let start = (camera / TILE).floor() as i32 - 1;
    for y in 0..15 {
        for x in start..start + (view.x / TILE).ceil() as i32 + 3 {
            let mut py = y as f32 * TILE;
            if let Some((bx, by, timer)) = game.bumped
                && bx == x
                && by == y
            {
                py -= (timer / 0.18 * std::f32::consts::PI).sin() * 4.0;
            }
            tile(
                game,
                game.level.tile(x, y),
                x,
                y,
                x as f32 * TILE - camera,
                py,
            );
        }
    }
    for c in &game.level.coins {
        let x = c.pos.x - camera;
        if !c.collected
            && (-12.0..view.x).contains(&x)
            && (game.phase != Phase::Title || c.pos.y > 160.0)
        {
            let y = c.pos.y + (game.time * 4.0 + c.pos.x * 0.05).sin();
            if c.special {
                challenge_coin(x, y, true);
            } else {
                coin(x, y, game.time + c.pos.x * 0.01);
            }
        }
    }
    for platform in &game.level.platforms {
        let bounds = platform.rect();
        if bounds.x + bounds.w < camera || bounds.x > camera + view.x || bounds.y > HEIGHT {
            continue;
        }
        let x = bounds.x - camera;
        if !platform.crumble {
            rect(
                platform.origin.x - camera - platform.travel.x,
                platform.origin.y + 2.0,
                platform.width + platform.travel.x * 2.0,
                1.0,
                color_u8!(92, 119, 125, 255),
            );
        }
        let shaking = platform.touched.is_some_and(|time| time < 0.6);
        let shift = if shaking {
            (game.level.clock * 45.0).sin().signum()
        } else {
            0.0
        };
        rect(x + shift, bounds.y, bounds.w, 6.0, INK);
        rect(
            x + 1.0 + shift,
            bounds.y,
            bounds.w - 2.0,
            2.0,
            if platform.crumble { ORANGE } else { CREAM },
        );
        for n in 0..(bounds.w as i32 / 8) {
            rect(
                x + n as f32 * 8.0 + 2.0 + shift,
                bounds.y + 3.0,
                5.0,
                2.0,
                if platform.crumble {
                    RED
                } else {
                    color_u8!(85, 165, 171, 255)
                },
            );
        }
    }
    for jet in &game.level.fire {
        let x = jet.pos.x - camera;
        rect(x, jet.pos.y - 4.0, 16.0, 4.0, INK);
        rect(
            x + 3.0,
            jet.pos.y - 3.0,
            10.0,
            2.0,
            if jet.warning(game.level.clock) || jet.hot(game.level.clock) {
                GOLD
            } else {
                RED
            },
        );
        if jet.hot(game.level.clock) {
            let flicker = (game.time * 30.0).sin() * 2.0;
            rect(x + 2.0, jet.pos.y - 32.0, 12.0, 29.0, RED);
            rect(
                x + 4.0,
                jet.pos.y - 30.0 + flicker,
                8.0,
                27.0 - flicker,
                ORANGE,
            );
            rect(x + 6.0, jet.pos.y - 24.0, 4.0, 21.0, GOLD);
        } else if jet.warning(game.level.clock) {
            rect(x + 6.0, jet.pos.y - 10.0, 4.0, 5.0, GOLD);
        }
    }
    for enemy in &game.level.enemies {
        let x = enemy.pos.x - camera;
        let y = enemy.pos.y;
        if !(-16.0..view.x).contains(&x) {
            continue;
        }
        if enemy.squished.is_some() {
            rect(x, y + 9.0, 14.0, 3.0, RED);
            rect(x + 2.0, y + 9.0, 10.0, 1.0, ORANGE);
        } else {
            if enemy.kind == EnemyKind::Hopper {
                rect(x + 2.0, y - 7.0, 3.0, 8.0, CREAM);
                rect(x + 9.0, y - 7.0, 3.0, 8.0, CREAM);
            } else if enemy.kind == EnemyKind::Flyer {
                let wing = (game.time * 20.0).sin() * 3.0;
                rect(x - 6.0, y + wing, 8.0, 3.0, CREAM);
                rect(x + 12.0, y - wing, 8.0, 3.0, CREAM);
            }
            sprite(
                &[
                    "....oooooo....",
                    "..oorhhhhroo..",
                    ".orhhhyhhhhhro",
                    ".orhhyhhhyhro.",
                    "orhhhyhhhhyhro",
                    "orrrrrrrrrrro.",
                    "oococrrcocoo..",
                    ".ococrrcoco...",
                    "..ossssssso...",
                    "...oooooo.....",
                ],
                x,
                y,
                enemy.vel.x.signum(),
            );
            let step = ((game.time * 7.0) as i32 % 2) as f32;
            rect(x + 1.0 + step, y + 10.0, 4.0, 2.0, INK);
            rect(x + 9.0 - step, y + 10.0, 4.0, 2.0, INK);
        }
    }
    player(game, camera);
    if game.stomp_chain > 1 {
        let label = format!("X{}", game.stomp_chain);
        let x = game.player.pos.x - camera - 2.0;
        let y = game.player.pos.y - 13.0;
        rect(x - 2.0, y - 2.0, text_width(&label, 1.0) + 4.0, 11.0, INK);
        text(&label, x, y, 1.0, GOLD);
    }
    for p in &game.particles {
        rect(
            p.pos.x - camera,
            p.pos.y,
            2.0,
            2.0,
            if p.gold { GOLD } else { CREAM },
        );
    }
}

pub fn draw_ui(game: &Game, muted: bool, view: Vec2) {
    if game.phase == Phase::LevelSelect {
        level_select(game, view);
    } else if game.phase == Phase::Title {
        title(game, muted, view);
    } else {
        hud(game, muted, view);
        match game.phase {
            Phase::Paused => panel(
                "TAKE A BREATHER",
                match game.mode {
                    Mode::Campaign => "YOUR ADVENTURE CAN WAIT.",
                    Mode::TimeTrial => "ONE ATTEMPT. NO CHECKPOINTS.",
                    Mode::Arcade => "THREE LIVES FOR ALL SIXTEEN LEVELS.",
                },
                if game.mode == Mode::Arcade {
                    "P RESUME  R NEW RUN  L LEVELS"
                } else {
                    "P RESUME   R RETRY   L LEVELS"
                },
                game,
                view,
            ),
            Phase::GameOver => panel(
                "ONE MORE TRY?",
                &if game.mode == Mode::Arcade {
                    format!("ARCADE BEST {:06}", game.progress.arcade_best)
                } else {
                    "EVERY GREAT JUMP STARTS SOMEWHERE.".into()
                },
                "ENTER RETRY    L LEVELS",
                game,
                view,
            ),
            Phase::Won => panel(
                "YOU DID IT!",
                &if game.mode == Mode::Arcade {
                    format!("ARCADE BEST {:06}", game.progress.arcade_best)
                } else {
                    "SIXTEEN LEVELS. ONE LITTLE LEGEND.".into()
                },
                if game.mode == Mode::Arcade {
                    "C NEW ARCADE    L LEVELS"
                } else {
                    "ENTER CONTINUE    L LEVELS"
                },
                game,
                view,
            ),
            Phase::StageClear => panel(
                if game
                    .finish
                    .as_ref()
                    .is_some_and(|result| result.personal_best)
                {
                    "NEW BEST!"
                } else {
                    "NICE RUN!"
                },
                game.level.name,
                if game.mode == Mode::TimeTrial {
                    "ENTER / R RETRY   L LEVELS"
                } else if game.stage + 1 == STAGE_COUNT {
                    "HOME, SWEET HOME."
                } else {
                    "ON TO THE NEXT ADVENTURE..."
                },
                game,
                view,
            ),
            _ => {}
        }
    }
    if let Some(notice) = game.save_notice {
        rect(0.0, view.y - 10.0, view.x, 10.0, INK);
        centered(notice, view.y - 9.0, 1.0, GOLD, view.x);
    }
}

fn level_select(game: &Game, view: Vec2) {
    rect(0.0, 0.0, view.x, view.y, INK);
    centered(
        if game.mode == Mode::TimeTrial {
            "TIME TRIAL"
        } else {
            "CHOOSE YOUR LEVEL"
        },
        20.0,
        2.0,
        GOLD,
        view.x,
    );
    let left = (view.x - 344.0) / 2.0;
    for stage in 0..STAGE_COUNT {
        let x = left + (stage % 4) as f32 * 88.0;
        let y = 51.0 + (stage / 4) as f32 * 32.0;
        let unlocked = stage <= game.progress.unlocked();
        rect(
            x,
            y,
            80.0,
            27.0,
            if stage == game.selected_stage {
                GOLD
            } else {
                color_u8!(64, 85, 84, 255)
            },
        );
        let label = if unlocked {
            crate::levels::world_label(stage)
        } else {
            "LOCKED".into()
        };
        text(
            &label,
            x + 8.0,
            y + 5.0,
            1.0,
            if stage == game.selected_stage {
                INK
            } else {
                CREAM
            },
        );
        if unlocked {
            let record = &game.progress.levels[stage];
            for (i, (label, earned)) in [
                ("C", record.cleared),
                ("S", record.speed_medal),
                ("G", record.treasure_medal),
            ]
            .into_iter()
            .enumerate()
            {
                text(
                    if earned { label } else { "-" },
                    x + 8.0 + i as f32 * 21.0,
                    y + 16.0,
                    1.0,
                    if stage == game.selected_stage {
                        INK
                    } else if earned {
                        GOLD
                    } else {
                        color_u8!(130, 150, 146, 255)
                    },
                );
            }
        }
    }
    let course = &crate::levels::COURSES[game.selected_stage];
    let record = &game.progress.levels[game.selected_stage];
    centered(
        crate::levels::WORLD_NAMES[game.selected_stage / 4],
        39.0,
        1.0,
        CREAM,
        view.x,
    );
    centered(course.name, 181.0, 1.0, CREAM, view.x);
    centered(
        &format!(
            "BEST {}  SCORE {}  PAR {}",
            record
                .best_ms
                .map(time_label)
                .unwrap_or_else(|| "--.--".into()),
            record.high_score,
            time_label(course.par_ms)
        ),
        194.0,
        1.0,
        GOLD,
        view.x,
    );
    centered("C CLEAR  S SPEED  G THREE GEMS", 207.0, 1.0, CREAM, view.x);
    centered(
        if game.mode == Mode::TimeTrial {
            "T SWITCH TO CAMPAIGN"
        } else {
            "T SWITCH TO TIME TRIAL"
        },
        218.0,
        1.0,
        GOLD,
        view.x,
    );
    centered(
        "ARROWS SELECT  ENTER PLAY  ESC BACK",
        view.y - 11.0,
        1.0,
        CREAM,
        view.x,
    );
}
