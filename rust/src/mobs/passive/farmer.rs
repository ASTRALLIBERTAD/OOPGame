use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;

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
    wander_radius: f32,

    #[export]
    wander_interval: f64,

    #[export]
    flee_duration: f64,

    #[export]
    harvest_interval: f64,

    #[export]
    #[var(get = get_trust, set = set_trust)]
    trust: i32,

    mob_state: MobState,
    home_position: Vector2,
    wander_target: Vector2,
    wander_timer: f64,
    flee_target: Option<Vector2>,
    flee_timer: f64,
    harvest_timer: f64,

    has_quest: bool,

    #[export]
    quest_kill_target: i32,
    quest_kills: i32,
    quest_active: bool,
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
            wander_radius: 200.0,
            wander_interval: 5.0,
            flee_duration: 3.5,
            harvest_interval: 30.0,
            trust: 0,
            mob_state: MobState::Idle,
            home_position: Vector2::ZERO,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            flee_target: None,
            flee_timer: 0.0,
            harvest_timer: 0.0,
            has_quest: true,
            quest_kill_target: 3,
            quest_kills: 0,
            quest_active: false,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("neutral");
        self.base_mut().add_to_group("civilian");
        let pos = self.base_mut().get_global_position();
        self.wander_target = pos;
        self.home_position = pos;
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.harvest_timer += delta;
        if self.harvest_timer >= self.harvest_interval {
            self.harvest_timer = 0.0;
            let pos = self.base_mut().get_global_position();
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("food_ready")),
                    Variant::from(pos),
                ],
            );
        }

        if self.mob_state == MobState::Fleeing {
            self.flee_timer += delta;
            if self.flee_timer >= self.flee_duration {
                self.flee_timer = 0.0;
                self.flee_target = None;
                self.mob_state = MobState::Idle;
                self.wander();
            } else if let Some(flee_pos) = self.flee_target {
                self.move_toward(flee_pos, self.flee_speed);
                return;
            }
        }

        if let Some(threat_pos) = self.nearest_enemy_position() {
            if let Some(priest_pos) = self.nearest_priest_position() {
                self.flee_to(priest_pos);
            } else {
                self.flee(threat_pos);
            }
            return;
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
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[Variant::from(GString::from("civilian_killed"))],
            );
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
        let angle = (godot::global::randf() * std::f64::consts::TAU) as f32;
        let dist = godot::global::randf() as f32 * self.wander_radius;
        let offset = Vector2::new(angle.cos() * dist, angle.sin() * dist);
        self.wander_target = self.home_position + offset;
    }

    fn flee(&mut self, from: Vector2) {
        let pos = self.base_mut().get_global_position();
        let away = (pos - from).normalized();
        self.flee_target = Some(pos + away * 250.0);
        self.flee_timer = 0.0;
        self.mob_state = MobState::Fleeing;
        godot_print!("Magsasaka is fleeing!");
    }
}

#[godot_api]
impl Farmer {
    #[signal]
    fn quest_completed();

    #[signal]
    fn civilian_killed();

    #[func]
    pub fn on_enemy_killed_nearby(&mut self) {
        godot_print!(
            "on_enemy_killed_nearby: has_quest={} quest_active={} kills={}/{}",
            self.has_quest,
            self.quest_active,
            self.quest_kills,
            self.quest_kill_target
        );
        if !self.has_quest || !self.quest_active {
            return;
        }
        self.quest_kills += 1;
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        if self.quest_kills >= self.quest_kill_target {
            self.on_quest_complete();
        } else {
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from(
                        format!(
                            "Magsasaka: 'Thank you! {}/{} enemies cleared.'",
                            self.quest_kills, self.quest_kill_target
                        )
                        .as_str(),
                    )),
                ],
            );
        }
    }

    fn flee_to(&mut self, destination: Vector2) {
        self.flee_target = Some(destination);
        self.flee_timer = 0.0;
        self.mob_state = MobState::Fleeing;
        godot_print!("Magsasaka runs to the Priest!");
    }

    fn nearest_priest_position(&mut self) -> Option<Vector2> {
        let my_pos = self.base_mut().get_global_position();
        let healers = self.base_mut().get_tree().get_nodes_in_group("healer");
        let mut nearest: Option<(f32, Vector2)> = None;
        for node in healers.iter_shared() {
            if let Ok(body) = node.try_cast::<CharacterBody2D>() {
                let pos = body.get_global_position();
                let dist = my_pos.distance_to(pos);
                if nearest.is_none_or(|(d, _)| dist < d) {
                    nearest = Some((dist, pos));
                }
            }
        }
        nearest.map(|(_, pos)| pos)
    }

    fn nearest_enemy_position(&mut self) -> Option<Vector2> {
        let my_pos = self.base_mut().get_global_position();
        let enemies = self.base_mut().get_tree().get_nodes_in_group("enemy");
        let mut nearest: Option<(f32, Vector2)> = None;
        for enemy in enemies.iter_shared() {
            if let Ok(body) = enemy.try_cast::<CharacterBody2D>() {
                let epos = body.get_global_position();
                let dist = my_pos.distance_to(epos);
                if dist <= self.fear_radius && nearest.is_none_or(|(d, _)| dist < d) {
                    nearest = Some((dist, epos));
                }
            }
        }
        nearest.map(|(_, pos)| pos)
    }

    fn move_toward(&mut self, target: Vector2, speed: f32) {
        let pos = self.base_mut().get_global_position();
        if pos.distance_to(target) < 6.0 {
            if speed == self.wander_speed {
                self.wander_timer = self.wander_interval;
            }
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
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        if self.has_quest {
            self.quest_active = true;
            self.base_mut().emit_signal("quest_taken", &[]);
            event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from(
                    format!(
                        "Magsasaka: 'Please help us. Kill {}/{} enemies near our farm. Trust: {}/5'",
                        self.quest_kills, self.quest_kill_target, self.trust
                    ).as_str()
                )),
            ],
        );
        } else {
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from(
                        format!(
                            "Magsasaka: 'Thank you for your help. Trust: {}/5'",
                            self.trust
                        )
                        .as_str(),
                    )),
                ],
            );
        }
    }

    #[func]
    pub fn on_quest_complete(&mut self) {
        self.has_quest = false;
        self.quest_active = false;
        self.quest_kills = 0;
        self.trust = (self.trust + 1).min(5);
        self.base_mut().emit_signal("quest_completed", &[]);

        let pos = self.base_mut().get_global_position();
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");

        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("item_dropped")),
                Variant::from(GString::from("palay")),
                Variant::from(pos),
            ],
        );

        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("piso_dropped")),
                Variant::from(50_i32),
                Variant::from(pos),
            ],
        );

        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from(
                    format!("Magsasaka: 'We are grateful. Trust: {}/5'", self.trust).as_str(),
                )),
            ],
        );
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
