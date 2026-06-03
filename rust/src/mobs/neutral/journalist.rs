use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;

use crate::entity::{Entity, HostileBehavior, MobState, NeutralBehavior};
use crate::rustplayer::Rustplayer;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Journalist {
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
    intel_interval: f64,

    #[export]
    publish_interval: f64,

    #[export]
    blackout_duration: f64,

    #[export]
    hostile_corruption_threshold: i32,

    #[export]
    #[var(get = get_trust, set = set_trust)]
    trust: i32,

    #[export]
    #[var(get = get_intel_count)]
    intel_count: i32,

    #[export]
    #[var(get = get_corruption_level, set = set_corruption_level)]
    corruption_level: i32,

    mob_state: MobState,
    is_hostile: bool,
    home_position: Vector2,
    wander_target: Vector2,
    wander_timer: f64,
    flee_target: Option<Vector2>,
    flee_timer: f64,
    intel_timer: f64,
    publish_timer: f64,
    boss_exposed: bool,
    blackout_active: bool,
    blackout_timer: f64,
}

#[godot_api]
impl ICharacterBody2D for Journalist {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: 18,
            wander_speed: 45.0,
            flee_speed: 110.0,
            fear_radius: 140.0,
            wander_radius: 180.0,
            wander_interval: 5.0,
            flee_duration: 3.0,
            intel_interval: 20.0,
            publish_interval: 45.0,
            blackout_duration: 30.0,
            hostile_corruption_threshold: 8,
            trust: 0,
            intel_count: 0,
            corruption_level: 0,
            mob_state: MobState::Idle,
            is_hostile: false,
            home_position: Vector2::ZERO,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            flee_target: None,
            flee_timer: 0.0,
            intel_timer: 0.0,
            publish_timer: 0.0,
            boss_exposed: false,
            blackout_active: false,
            blackout_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("neutral");
        self.base_mut().add_to_group("journalist");
        let pos = self.base_mut().get_global_position();
        self.wander_target = pos;
        self.home_position = pos;
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.tick_intel(delta);
        self.tick_publish(delta);
        self.tick_blackout(delta);

        if self.is_hostile {
            self.process_hostile(delta);
            return;
        }

        if self.mob_state == MobState::Fleeing {
            self.flee_timer += delta;
            if self.flee_timer >= self.flee_duration {
                self.flee_timer = 0.0;
                self.flee_target = None;
                self.mob_state = MobState::Idle;
                self.pick_wander_target();
            } else if let Some(fp) = self.flee_target {
                self.move_toward(fp, self.flee_speed);
                return;
            }
        }

        if let Some(threat) = self.nearest_threat_position() {
            self.flee_from(threat);
            return;
        }

        self.wander_timer += delta;
        if self.wander_timer >= self.wander_interval {
            self.wander_timer = 0.0;
            self.pick_wander_target();
        }
        let target = self.wander_target;
        self.move_toward(target, self.wander_speed);
    }
}

impl Entity for Journalist {
    fn take_damage(&mut self, amount: i32) {
        if !self.is_alive() {
            return;
        }
        self.health = (self.health - amount).max(0);

        if !self.is_hostile && self.corruption_level >= self.hostile_corruption_threshold {
            self.become_hostile();
        }

        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from("The journalist has been silenced.")),
                ],
            );
            event_bus.call(
                "emit_signal",
                &[Variant::from(GString::from("civilian_killed"))],
            );
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).clamp(0, 18);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl NeutralBehavior for Journalist {
    fn interact(&self) -> &'static str {
        "dialogue.journalist.greet"
    }

    fn become_hostile(&mut self) {
        if self.is_hostile {
            return;
        }
        self.is_hostile = true;
        self.base_mut().remove_from_group("neutral");
        self.base_mut().add_to_group("enemy");
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from("Mamamahayag: 'I've had enough!'")),
            ],
        );
    }
}

impl HostileBehavior for Journalist {
    fn aggro(&mut self, _target: Vector2) {
        self.mob_state = MobState::Aggro;
    }

    fn chase(&mut self, target: Vector2, speed: f32) {
        let pos = self.base_mut().get_global_position();
        let dir = (target - pos).normalized();
        self.sprite.set_flip_h(dir.x < 0.0);
        self.base_mut().set_velocity(dir * speed);
        self.base_mut().move_and_slide();
    }

    fn attack(&mut self, target: &mut dyn Entity) {
        target.take_damage(2);
    }
}

#[godot_api]
impl Journalist {
    #[signal]
    fn expose_boss();

    #[func]
    pub fn on_interact(&mut self) {
        if self.is_hostile {
            return;
        }

        self.trust = (self.trust + 1).min(5);
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");

        if self.trust >= 3 {
            let pos = self.base_mut().get_global_position();
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("item_dropped")),
                    Variant::from(GString::from("intel_document")),
                    Variant::from(pos),
                ],
            );
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from(
                        format!(
                            "Mamamahayag: 'Here is what I know. Trust: {}/5'",
                            self.trust
                        )
                        .as_str(),
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
                            "Mamamahayag: 'I'm watching everything. Trust: {}/5'",
                            self.trust
                        )
                        .as_str(),
                    )),
                ],
            );
        }

        if self.trust >= 5 && !self.boss_exposed {
            self.boss_exposed = true;
            self.base_mut().emit_signal("expose_boss", &[]);
            self.trigger_press_blackout();
        }
    }

    #[func]
    pub fn press_discount(&self) -> f32 {
        (self.trust as f32 / 5.0) * 0.3
    }

    #[func]
    pub fn is_blackout_active(&self) -> bool {
        self.blackout_active
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, 18);
    }

    #[func]
    pub fn get_trust(&self) -> i32 {
        self.trust
    }

    #[func]
    pub fn set_trust(&mut self, trust: i32) {
        self.trust = trust.clamp(0, 5);
    }

    #[func]
    pub fn get_intel_count(&self) -> i32 {
        self.intel_count
    }

    #[func]
    pub fn get_corruption_level(&self) -> i32 {
        self.corruption_level
    }

    #[func]
    pub fn set_corruption_level(&mut self, level: i32) {
        self.corruption_level = level.clamp(0, 10);
    }

    fn trigger_press_blackout(&mut self) {
        self.blackout_active = true;
        self.blackout_timer = 0.0;
        let dur = self.blackout_duration;
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from(
                    format!(
                        "Mamamahayag: 'Boss regens from corruption tiles — cleanse them! Blackout for {}s.'",
                        dur as i32
                    )
                    .as_str(),
                )),
            ],
        );
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("press_blackout")),
                Variant::from(dur),
            ],
        );
    }

    fn tick_blackout(&mut self, delta: f64) {
        if !self.blackout_active {
            return;
        }
        self.blackout_timer += delta;
        if self.blackout_timer >= self.blackout_duration {
            self.blackout_active = false;
            self.blackout_timer = 0.0;
        }
    }

    fn tick_intel(&mut self, delta: f64) {
        let my_pos = self.base().get_global_position();
        let has_nearby_enemy = self
            .base_mut()
            .get_tree()
            .get_nodes_in_group("enemy")
            .iter_shared()
            .any(|node| {
                node.try_cast::<CharacterBody2D>()
                    .map(|b| b.get_global_position().distance_to(my_pos) < 300.0)
                    .unwrap_or(false)
            });

        let rate = if has_nearby_enemy {
            self.intel_interval * 0.5
        } else {
            self.intel_interval
        };

        self.intel_timer += delta;
        if self.intel_timer >= rate {
            self.intel_timer = 0.0;
            self.intel_count += 1;
        }
    }

    fn tick_publish(&mut self, delta: f64) {
        if self.intel_count < 3 {
            return;
        }
        self.publish_timer += delta;
        if self.publish_timer >= self.publish_interval {
            self.publish_timer = 0.0;
            let count = self.intel_count;
            self.intel_count = 0;
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("article_published")),
                    Variant::from(count),
                ],
            );
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from(
                        format!("Mamamahayag published an article! Intel: {}", count).as_str(),
                    )),
                ],
            );
        }
    }

    fn process_hostile(&mut self, delta: f64) {
        let my_pos = self.base_mut().get_global_position();
        let Some(player_node) = self
            .base_mut()
            .get_tree()
            .get_nodes_in_group("player")
            .get(0)
        else {
            return;
        };
        let Ok(player_gd) = player_node.try_cast::<CharacterBody2D>() else {
            return;
        };
        let player_pos = player_gd.get_global_position();
        let distance = my_pos.distance_to(player_pos);

        if distance <= 30.0 {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                player.bind_mut().take_damage(2);
            }
            let away = (my_pos - player_pos).normalized();
            self.flee_target = Some(my_pos + away * 300.0);
            self.flee_timer = 0.0;
            self.mob_state = MobState::Fleeing;
            self.is_hostile = false;
            self.base_mut().remove_from_group("enemy");
            self.base_mut().add_to_group("neutral");
        } else {
            self.aggro(player_pos);
            self.chase(player_pos, self.flee_speed * 0.8);
        }
        let _ = delta;
    }

    fn flee_from(&mut self, from: Vector2) {
        let pos = self.base_mut().get_global_position();
        let away = (pos - from).normalized();
        self.flee_target = Some(pos + away * 250.0);
        self.flee_timer = 0.0;
        self.mob_state = MobState::Fleeing;
    }

    fn nearest_threat_position(&mut self) -> Option<Vector2> {
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

    fn pick_wander_target(&mut self) {
        let angle = (godot::global::randf() * std::f64::consts::TAU) as f32;
        let dist = godot::global::randf() as f32 * self.wander_radius;
        let offset = Vector2::new(angle.cos() * dist, angle.sin() * dist);
        self.wander_target = self.home_position + offset;
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
}
