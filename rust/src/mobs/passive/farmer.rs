use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, MobState, PassiveBehavior};

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Farmer {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    wander_speed: f32,

    #[export]
    flee_speed: f32,

    #[export]
    fear_radius: f32,

    #[export]
    harvest_interval: f64,

    #[export]
    #[var(get = get_trust, set = set_trust)]
    trust: i32,

    mob_state: MobState,

    wander_target: Vector2,
    wander_timer: f64,
    wander_interval: f64,

    flee_target: Option<Vector2>,
    flee_timer: f64,
    flee_duration: f64,

    harvest_timer: f64,
    has_quest: bool,
}

#[godot_api]
impl ICharacterBody2D for Farmer {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: 20,
            wander_speed: 40.0,
            flee_speed: 95.0,
            fear_radius: 160.0,
            harvest_interval: 30.0,
            trust: 0,
            mob_state: MobState::Idle,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            wander_interval: 5.0,
            flee_target: None,
            flee_timer: 0.0,
            flee_duration: 3.5,
            harvest_timer: 0.0,
            has_quest: true,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("neutral");
        self.base_mut().add_to_group("civilian");
        let global_position = self.base_mut().get_global_position();
        self.wander_target = global_position;
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.harvest_timer += delta;
        if self.harvest_timer >= self.harvest_interval {
            self.harvest_timer = 0.0;
            self.base_mut().emit_signal("food_ready", &[]);
            godot_print!("Magsasaka harvested produce.");
        }

        if let Some(threat_pos) = self.nearest_enemy_position() {
            self.flee(threat_pos);
            return;
        }

        if self.mob_state == MobState::Fleeing {
            self.flee_timer += delta;
            if self.flee_timer >= self.flee_duration {
                self.flee_timer = 0.0;
                self.flee_target = None;
                self.mob_state = MobState::Idle;
            } else if let Some(flee_pos) = self.flee_target {
                self.move_toward(flee_pos, self.flee_speed);
                return;
            }
        }

        self.wander_timer += delta;
        if self.wander_timer >= self.wander_interval {
            self.wander_timer = 0.0;
            self.wander();
        }
        let target = self.wander_target;
        self.move_toward(target, self.wander_speed);
    }
}

impl Entity for Farmer {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.base_mut().emit_signal("civilian_killed", &[]);
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(20);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl PassiveBehavior for Farmer {
    fn wander(&mut self) {
        let pos = self.base_mut().get_global_position();
        let offset = Vector2::new((pseudo_rand() - 0.5) * 200.0, (pseudo_rand() - 0.5) * 200.0);
        self.wander_target = pos + offset;
    }

    fn flee(&mut self, from: Vector2) {
        let pos = self.base_mut().get_global_position();
        let away = (pos - from).normalized();
        let flee_dest = pos + away * 250.0;
        self.flee_target = Some(flee_dest);
        self.flee_timer = 0.0;
        self.mob_state = MobState::Fleeing;
        self.move_toward(flee_dest, self.flee_speed);
        godot_print!("Magsasaka is fleeing!");
    }
}

#[godot_api]
impl Farmer {
    #[signal]
    fn food_ready();

    #[signal]
    fn quest_taken();

    #[signal]
    fn quest_completed();

    #[signal]
    fn civilian_killed();

    fn nearest_enemy_position(&mut self) -> Option<Vector2> {
        let my_pos = self.base_mut().get_global_position();
        let enemies = self.base_mut().get_tree().get_nodes_in_group("enemy");

        let mut nearest: Option<(f32, Vector2)> = None;
        for enemy in enemies.iter_shared() {
            if let Ok(body) = enemy.try_cast::<CharacterBody2D>() {
                let epos = body.get_global_position();
                let dist = my_pos.distance_to(epos);
                if dist <= self.fear_radius
                    && nearest.is_none_or(|(d, _)| dist < d) {
                        nearest = Some((dist, epos));
                    }
            }
        }
        nearest.map(|(_, pos)| pos)
    }

    fn move_toward(&mut self, target: Vector2, speed: f32) {
        let pos = self.base_mut().get_global_position();
        if pos.distance_to(target) < 6.0 {
            self.base_mut().set_velocity(Vector2::ZERO);
        } else {
            let dir = (target - pos).normalized();
            self.sprite.set_flip_h(dir.x < 0.0);
            self.base_mut().set_velocity(dir * speed);
        }
        self.base_mut().move_and_slide();
    }

    #[func]
    pub fn on_interact(&mut self) {
        if self.has_quest {
            self.base_mut().emit_signal("quest_taken", &[]);
            godot_print!("Magsasaka: 'Please help us. Trust: {}/5'", self.trust);
        } else {
            godot_print!(
                "Magsasaka: 'Thank you for your help. Trust: {}/5'",
                self.trust
            );
        }
    }

    #[func]
    pub fn on_quest_complete(&mut self) {
        self.has_quest = false;
        self.trust = (self.trust + 1).min(5);
        self.base_mut().emit_signal("quest_completed", &[]);
        godot_print!("Magsasaka: 'We are grateful. Trust: {}/5'", self.trust);
    }

    #[func]
    pub fn trade_discount(&self) -> f32 {
        (self.trust as f32 / 5.0) * 0.5
    }

    #[func]
    pub fn set_trust(&mut self, trust: i32) {
        self.trust = trust.clamp(0, 5);
    }

    #[func]
    pub fn get_trust(&self) -> i32 {
        self.trust
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, 20);
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }
}

fn pseudo_rand() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (t % 10_000) as f32 / 10_000.0
}
