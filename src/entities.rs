use macroquad::prelude::{Rect, Vec2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Beetle,
    Hopper,
    Flyer,
}

pub struct Enemy {
    pub pos: Vec2,
    pub vel: Vec2,
    pub squished: Option<f32>,
    pub kind: EnemyKind,
    pub origin: Vec2,
    pub clock: f32,
}

impl Enemy {
    pub fn new(x: f32, y: f32) -> Self {
        Self::of_kind(x, y, EnemyKind::Beetle)
    }
    pub fn of_kind(x: f32, y: f32, kind: EnemyKind) -> Self {
        Self {
            pos: vec2(x, y),
            vel: vec2(-26.0, 0.0),
            squished: None,
            kind,
            origin: vec2(x, y),
            clock: 0.0,
        }
    }
    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, 14.0, 12.0)
    }
}

pub struct Platform {
    pub origin: Vec2,
    pub pos: Vec2,
    pub previous: Vec2,
    pub width: f32,
    pub travel: Vec2,
    pub crumble: bool,
    pub touched: Option<f32>,
    clock: f32,
}

impl Platform {
    pub fn moving(x: f32, y: f32, width: f32, travel: Vec2) -> Self {
        Self {
            origin: vec2(x, y),
            pos: vec2(x, y),
            previous: vec2(x, y),
            width,
            travel,
            crumble: false,
            touched: None,
            clock: 0.0,
        }
    }
    pub fn crumbling(x: f32, y: f32, width: f32) -> Self {
        Self {
            crumble: true,
            ..Self::moving(x, y, width, Vec2::ZERO)
        }
    }
    pub fn solid(&self) -> bool {
        self.touched.is_none_or(|time| time < 0.6)
    }
    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.width, 6.0)
    }
    pub fn step(&mut self, dt: f32) {
        self.previous = self.pos;
        self.clock += dt;
        if self.crumble {
            if let Some(time) = &mut self.touched {
                *time += dt;
                if *time >= 3.0 {
                    self.touched = None;
                    self.pos = self.origin;
                    self.previous = self.origin;
                } else if *time >= 0.6 {
                    self.pos.y = self.origin.y + 380.0 * (*time - 0.6).powi(2);
                }
            }
        } else {
            self.pos = self.origin + self.travel * (self.clock * 1.5).sin();
        }
    }
    pub fn land(&mut self) {
        if self.crumble && self.touched.is_none() {
            self.touched = Some(0.0);
        }
    }
}

pub struct FireJet {
    pub pos: Vec2,
    pub offset: f32,
}
impl FireJet {
    pub fn new(x: f32, floor_y: f32, offset: f32) -> Self {
        Self {
            pos: vec2(x, floor_y),
            offset,
        }
    }
    pub fn cycle(&self, time: f32) -> f32 {
        (time + self.offset).rem_euclid(3.6)
    }
    pub fn hot(&self, time: f32) -> bool {
        (1.2..2.4).contains(&self.cycle(time))
    }
    pub fn warning(&self, time: f32) -> bool {
        (0.7..1.2).contains(&self.cycle(time))
    }
    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x + 2.0, self.pos.y - 32.0, 12.0, 32.0)
    }
}
