use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;

use crate::entity::{Entity, HostileBehavior, MobState, NeutralBehavior};
use crate::rustplayer::Rustplayer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudentAlignment {
    Neutral,
    Allied,
    Radicalized,
}

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Student {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    #[var(get = get_trust, set = set_trust)]
    trust: i32,

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
    ally_trust_threshold: i32,

    #[export]
    radicalize_range: f32,

    #[export]
    rally_interval: f64,

    #[export]
    rally_radius: f32,

    #[export]
    rally_speed_bonus: f32,

    #[export]
    rally_duration: f64,

    #[export]
    attack_damage: i32,

    #[export]
    attack_cooldown: f64,

    alignment: StudentAlignment,
    mob_state: MobState,
    home_position: Vector2,
    wander_target: Vector2,
    wander_timer: f64,
    flee_target: Option<Vector2>,
    flee_timer: f64,
    radicalize_check_timer: f64,
    rally_timer: f64,
    can_slash: bool,
    slash_timer: f64,
}

#[godot_api]
impl ICharacterBody2D for Student {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: 22,
            trust: 0,
            wander_speed: 55.0,
            flee_speed: 125.0,
            fear_radius: 150.0,
            wander_radius: 160.0,
            wander_interval: 4.5,
            flee_duration: 3.5,
            ally_trust_threshold: 3,
            radicalize_range: 120.0,
            rally_interval: 5.0,
            rally_radius: 100.0,
            rally_speed_bonus: 40.0,
            rally_duration: 4.0,
            attack_damage: 4,
            attack_cooldown: 1.3,
            alignment: StudentAlignment::Neutral,
            mob_state: MobState::Idle,
            home_position: Vector2::ZERO,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            flee_target: None,
            flee_timer: 0.0,
            radicalize_check_timer: 0.0,
            rally_timer: 0.0,
            can_slash: true,
            slash_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("neutral");
        self.base_mut().add_to_group("student");
        let pos = self.base_mut().get_global_position();
        self.wander_target = pos;
        self.home_position = pos;
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }
        match self.alignment {
            StudentAlignment::Radicalized => self.process_radicalized(delta),
            StudentAlignment::Allied => self.process_allied(delta),
            StudentAlignment::Neutral => self.process_neutral(delta),
        }
    }
}

impl Entity for Student {
    fn take_damage(&mut self, amount: i32) {
        if !self.is_alive() {
            return;
        }
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
        self.health = (self.health + amount).clamp(0, 22);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl NeutralBehavior for Student {
    fn interact(&self) -> &'static str {
        "dialogue.student.greet"
    }

    fn become_hostile(&mut self) {
        if self.alignment == StudentAlignment::Radicalized {
            return;
        }
        self.alignment = StudentAlignment::Radicalized;
        self.mob_state = MobState::Aggro;
        self.base_mut().remove_from_group("neutral");
        self.base_mut().remove_from_group("student");
        self.base_mut().add_to_group("enemy");
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from("Estudyante has been radicalized!")),
            ],
        );
    }
}

impl HostileBehavior for Student {
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
        if !self.can_slash {
            return;
        }
        self.can_slash = false;
        self.slash_timer = 0.0;
        target.take_damage(self.attack_damage);
    }
}

#[godot_api]
impl Student {
    #[signal]
    fn student_allied();

    #[func]
    pub fn on_interact(&mut self) {
        if self.alignment == StudentAlignment::Radicalized {
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from("Estudyante ignores you.")),
                ],
            );
            return;
        }

        self.trust = (self.trust + 1).min(5);
        let pos = self.base_mut().get_global_position();
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");

        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("item_dropped")),
                Variant::from(GString::from("pamphlet")),
                Variant::from(pos),
            ],
        );

        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from(
                    format!("Estudyante: 'Trust: {}/5'", self.trust).as_str(),
                )),
            ],
        );

        let threshold = self.ally_trust_threshold;
        if self.trust >= threshold && self.alignment == StudentAlignment::Neutral {
            self.alignment = StudentAlignment::Allied;
            self.base_mut().remove_from_group("neutral");
            self.base_mut().add_to_group("civilian");
            self.base_mut().emit_signal("student_allied", &[]);
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from(
                        "Estudyante: 'I'm with you. Let's fight for real change.'",
                    )),
                ],
            );
        }
    }

    #[func]
    pub fn on_civilian_killed(&mut self) {
        if self.alignment != StudentAlignment::Neutral {
            return;
        }
        if self.trust == 0 {
            self.become_hostile();
        } else {
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from("Estudyante is shaken but holds.")),
                ],
            );
        }
    }

    #[func]
    pub fn get_alignment(&self) -> GString {
        match self.alignment {
            StudentAlignment::Neutral => GString::from("neutral"),
            StudentAlignment::Allied => GString::from("allied"),
            StudentAlignment::Radicalized => GString::from("radicalized"),
        }
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, 22);
    }

    #[func]
    pub fn get_trust(&self) -> i32 {
        self.trust
    }

    #[func]
    pub fn set_trust(&mut self, trust: i32) {
        self.trust = trust.clamp(0, 5);
    }

    fn process_neutral(&mut self, delta: f64) {
        self.radicalize_check_timer += delta;
        if self.radicalize_check_timer >= 1.0 {
            self.radicalize_check_timer = 0.0;
            self.check_troll_influence();
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

    fn process_allied(&mut self, delta: f64) {
        self.rally_timer += delta;
        if self.rally_timer >= self.rally_interval {
            self.rally_timer = 0.0;
            self.try_rally_player();
        }

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
        let dist = my_pos.distance_to(player_pos);

        if dist > 100.0 {
            self.move_toward(player_pos, self.wander_speed * 1.2);
        } else {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
        }
    }

    fn process_radicalized(&mut self, delta: f64) {
        if !self.can_slash {
            self.slash_timer += delta;
            if self.slash_timer >= self.attack_cooldown {
                self.can_slash = true;
                self.slash_timer = 0.0;
            }
        }

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
        let dist = my_pos.distance_to(player_pos);

        self.aggro(player_pos);
        self.chase(player_pos, self.wander_speed * 1.5);

        if dist <= 32.0 && self.can_slash {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                let dmg = self.attack_damage;
                player.bind_mut().take_damage(dmg);
            }
            self.can_slash = false;
            self.slash_timer = 0.0;
        }
    }

    fn try_rally_player(&mut self) {
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

        if my_pos.distance_to(player_pos) <= self.rally_radius {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                let bonus = self.rally_speed_bonus;
                let duration = self.rally_duration;
                player.bind_mut().apply_speed_bonus(bonus, duration);
            }
        }
    }

    fn check_troll_influence(&mut self) {
        if self.alignment != StudentAlignment::Neutral {
            return;
        }
        let my_pos = self.base_mut().get_global_position();
        let trolls = self.base_mut().get_tree().get_nodes_in_group("troll_pack");
        for troll_node in trolls.iter_shared() {
            if let Ok(body) = troll_node.try_cast::<CharacterBody2D>() {
                if my_pos.distance_to(body.get_global_position()) <= self.radicalize_range {
                    self.become_hostile();
                    return;
                }
            }
        }
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
