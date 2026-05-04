use godot::prelude::*;
use godot::obj::WithBaseField;
use crate::rustplayer::Rustplayer;
// backward-compatible
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MobState {
    Idle,
    Aggro,
    Fleeing,
    Dead,
}

// backward-compatible
#[allow(dead_code)]
pub trait Entity {
    fn update(&mut self);
    fn take_damage(&mut self, amount: f32);
    fn heal(&mut self, amount: f32);
    fn is_alive(&self) -> bool;
    fn position(&self) -> (f32, f32);
    fn set_position(&mut self, x: f32, y: f32);
}

// backward-compatible
impl Entity for Rustplayer {
    fn update(&mut self) {
        // backward-compatible: no-op, Rustplayer has its own process() method
    }

    fn take_damage(&mut self, amount: f32) {
        // backward-compatible: use public APIs through heart UI
        let amount_i = amount as i32;
        let mut heart = self.get_heart_ui();
        if heart.is_instance_valid() {
            heart.bind_mut().damage(amount_i);
        }
    }

    fn heal(&mut self, amount: f32) {
        // backward-compatible: use public APIs through heart UI
        let amount_i = amount as i32;
        let mut heart = self.get_heart_ui();
        if heart.is_instance_valid() {
            heart.bind_mut().heal(amount_i);
        }
    }

    fn is_alive(&self) -> bool {
        // backward-compatible: check heart UI current_health
        let heart = self.get_heart_ui();
        if heart.is_instance_valid() {
            heart.bind().current_health > 0
        } else {
            self.get_health() > 0
        }
    }

    fn position(&self) -> (f32, f32) {
        // backward-compatible: get position from base CharacterBody2D
        let pos = self.base().get_global_position();
        (pos.x, pos.y)
    }

    fn set_position(&mut self, x: f32, y: f32) {
        // backward-compatible: set position on base CharacterBody2D
        self.base_mut().set_global_position(Vector2::new(x, y));
    }
}

// backward-compatible
#[allow(dead_code)]
pub struct HostileMob {
    position: Vector2,
    health: i32,
    state: MobState,
    damage: f32,
    aggro_range: f32,
    attack_range: f32,
    speed: f32,
}

// backward-compatible
#[allow(dead_code)]
impl HostileMob {
    pub fn new(position: Vector2, health: i32, damage: f32, aggro_range: f32, attack_range: f32, speed: f32) -> Self {
        Self {
            position,
            health,
            state: MobState::Idle,
            damage,
            aggro_range,
            attack_range,
            speed,
        }
    }

    // backward-compatible
    pub fn detect_player(&self) -> bool {
        false
    }

    // backward-compatible
    pub fn attack(&mut self, target: &mut dyn Entity) {
        if self.state == MobState::Aggro {
            target.take_damage(self.damage);
        }
    }

    // backward-compatible
    pub fn chase(&mut self, target_pos: (f32, f32)) {
        if self.state == MobState::Aggro {
            let dx = target_pos.0 - self.position.x;
            let dy = target_pos.1 - self.position.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > self.speed * self.speed {
                let dist = dist_sq.sqrt();
                self.position.x += (dx / dist) * self.speed;
                self.position.y += (dy / dist) * self.speed;
            } else {
                self.position.x = target_pos.0;
                self.position.y = target_pos.1;
            }
        }
    }

    // backward-compatible
    fn update_aggro(&mut self, player_pos: (f32, f32)) {
        let dx = player_pos.0 - self.position.x;
        let dy = player_pos.1 - self.position.y;
        let dist = (dx * dx + dy * dy).sqrt();

        match self.state {
            MobState::Idle => {
                if dist < self.aggro_range {
                    self.state = MobState::Aggro;
                }
            }
            MobState::Aggro => {
                if dist > self.aggro_range * 2.0 {
                    self.state = MobState::Idle;
                } else if dist < self.attack_range {
                    self.attack_by_pos(player_pos);
                } else {
                    self.chase(player_pos);
                }
            }
            _ => {}
        }
    }

    // backward-compatible
    fn attack_by_pos(&mut self, _target_pos: (f32, f32)) {
    }
}

// backward-compatible
#[allow(dead_code)]
impl Entity for HostileMob {
    fn update(&mut self) {
    }

    fn take_damage(&mut self, amount: f32) {
        // backward-compatible
        self.health -= amount as i32;
        if self.health <= 0 {
            self.health = 0;
            self.state = MobState::Dead;
        }
    }

    fn heal(&mut self, amount: f32) {
        // backward-compatible
        if self.state != MobState::Dead {
            self.health += amount as i32;
        }
    }

    fn is_alive(&self) -> bool {
        // backward-compatible
        self.state != MobState::Dead && self.health > 0
    }

    fn position(&self) -> (f32, f32) {
        // backward-compatible
        (self.position.x, self.position.y)
    }

    fn set_position(&mut self, x: f32, y: f32) {
        // backward-compatible
        self.position.x = x;
        self.position.y = y;
    }
}

// backward-compatible
#[allow(dead_code)]
pub struct NeutralMob {
    position: Vector2,
    health: i32,
    state: MobState,
    is_hostile: bool,
    hostility_threshold: i32,
}

// backward-compatible
#[allow(dead_code)]
impl NeutralMob {
    pub fn new(position: Vector2, health: i32) -> Self {
        Self {
            position,
            health,
            state: MobState::Idle,
            is_hostile: false,
            hostility_threshold: health / 2,
        }
    }

    // backward-compatible
    pub fn interact(&self) -> &'static str {
        "Hello, traveler."
    }

    // backward-compatible
    pub fn become_hostile(&mut self) {
        self.is_hostile = true;
        self.state = MobState::Aggro;
    }
}

// backward-compatible
#[allow(dead_code)]
impl Entity for NeutralMob {
    fn update(&mut self) {
        // backward-compatible: calm default behavior
        if self.is_hostile && self.state == MobState::Idle {
            self.state = MobState::Aggro;
        }
    }

    fn take_damage(&mut self, amount: f32) {
        // backward-compatible: optional hostility if attacked
        self.health -= amount as i32;
        if self.health <= self.hostility_threshold && !self.is_hostile {
            self.become_hostile();
        }
        if self.health <= 0 {
            self.health = 0;
            self.state = MobState::Dead;
        }
    }

    fn heal(&mut self, amount: f32) {
        // backward-compatible
        if self.state != MobState::Dead {
            self.health += amount as i32;
        }
    }

    fn is_alive(&self) -> bool {
        // backward-compatible
        self.state != MobState::Dead && self.health > 0
    }

    fn position(&self) -> (f32, f32) {
        // backward-compatible
        (self.position.x, self.position.y)
    }

    fn set_position(&mut self, x: f32, y: f32) {
        // backward-compatible
        self.position.x = x;
        self.position.y = y;
    }
}

// backward-compatible
#[allow(dead_code)]
pub struct PassiveMob {
    position: Vector2,
    health: i32,
    state: MobState,
    wander_speed: f32,
    wander_target: Vector2,
    flee_speed: f32,
}

// backward-compatible
#[allow(dead_code)]
impl PassiveMob {
    pub fn new(position: Vector2, health: i32, wander_speed: f32, flee_speed: f32) -> Self {
        Self {
            position,
            health,
            state: MobState::Idle,
            wander_speed,
            wander_target: position,
            flee_speed,
        }
    }

    // backward-compatible
    pub fn wander(&mut self) {
        if self.state == MobState::Idle {
            let dx = self.wander_target.x - self.position.x;
            let dy = self.wander_target.y - self.position.y;
            if dx.abs() > 1.0 || dy.abs() > 1.0 {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq > self.wander_speed * self.wander_speed {
                    let dist = dist_sq.sqrt();
                    self.position.x += (dx / dist) * self.wander_speed;
                    self.position.y += (dy / dist) * self.wander_speed;
                } else {
                    self.position.x = self.wander_target.x;
                    self.position.y = self.wander_target.y;
                }
            }
        }
    }

    // backward-compatible
    pub fn flee(&mut self, from: (f32, f32)) {
        self.state = MobState::Fleeing;
        let dx = self.position.x - from.0;
        let dy = self.position.y - from.1;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq > 0.0 {
            let dist = dist_sq.sqrt();
            self.position.x += (dx / dist) * self.flee_speed;
            self.position.y += (dy / dist) * self.flee_speed;
        }
    }
}

// backward-compatible
#[allow(dead_code)]
impl Entity for PassiveMob {
    fn update(&mut self) {
        // backward-compatible: wander or flee
        match self.state {
            MobState::Idle => self.wander(),
            MobState::Fleeing => {
            }
            _ => {}
        }
    }

    fn take_damage(&mut self, amount: f32) {
        // backward-compatible: idle reaction to threats
        self.health -= amount as i32;
        if self.health <= 0 {
            self.health = 0;
            self.state = MobState::Dead;
        }
    }

    fn heal(&mut self, amount: f32) {
        // backward-compatible
        if self.state != MobState::Dead {
            self.health += amount as i32;
        }
    }

    fn is_alive(&self) -> bool {
        // backward-compatible
        self.state != MobState::Dead && self.health > 0
    }

    fn position(&self) -> (f32, f32) {
        // backward-compatible
        (self.position.x, self.position.y)
    }

    fn set_position(&mut self, x: f32, y: f32) {
        // backward-compatible
        self.position.x = x;
        self.position.y = y;
    }
}
