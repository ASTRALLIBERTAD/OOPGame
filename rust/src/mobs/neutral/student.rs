use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, HostileBehavior, MobState, NeutralBehavior};
use crate::rustplayer::Rustplayer;

const MAX_HP: i32 = 22;
const WANDER_SPEED: f32 = 55.0;
const FLEE_SPEED: f32 = 125.0;
const ALLY_TRUST_THRESHOLD: i32 = 3;
const MAX_TRUST: i32 = 5;
const FEAR_RADIUS: f32 = 150.0;
const FLEE_DURATION: f64 = 3.5;
const WANDER_RADIUS: f32 = 160.0;
const RADICALIZE_RANGE: f32 = 120.0;
const RADICALIZED_ATTACK: i32 = 4;
const RALLY_INTERVAL: f64 = 5.0;
const RALLY_RADIUS: f32 = 100.0;
const RALLY_SPEED_BONUS: f32 = 40.0;
const RALLY_DURATION: f64 = 4.0;
const ATTACK_COOLDOWN: f64 = 1.3;

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

    alignment: StudentAlignment,
    mob_state: MobState,

    wander_target: Vector2,
    wander_timer: f64,
    wander_interval: f64,

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
            health: MAX_HP,
            trust: 0,
            alignment: StudentAlignment::Neutral,
            mob_state: MobState::Idle,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            wander_interval: 4.5,
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
            self.base_mut().emit_signal("student_killed", &[]);
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).clamp(0, MAX_HP);
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
        godot_print!("Student radicalized.");
        self.base_mut().emit_signal("student_radicalized", &[]);
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
        target.take_damage(RADICALIZED_ATTACK);
    }
}

#[godot_api]
impl Student {
    #[signal]
    fn pamphlet_dropped(position: Vector2);

    #[signal]
    fn student_allied();

    #[signal]
    fn student_radicalized();

    #[signal]
    fn student_killed();

    #[signal]
    fn rally_triggered(speed_bonus: f32, duration: f64);

    #[func]
    pub fn on_interact(&mut self) {
        if self.alignment == StudentAlignment::Radicalized {
            godot_print!("Student is radicalized and ignores you.");
            return;
        }

        self.trust = (self.trust + 1).min(MAX_TRUST);

        let pos = self.base_mut().get_global_position();
        self.base_mut()
            .emit_signal("pamphlet_dropped", &[Variant::from(pos)]);

        godot_print!(
            "Student: '{}' (trust: {}/{})",
            self.interact(),
            self.trust,
            MAX_TRUST
        );

        if self.trust >= ALLY_TRUST_THRESHOLD && self.alignment == StudentAlignment::Neutral {
            self.alignment = StudentAlignment::Allied;
            self.base_mut().remove_from_group("neutral");
            self.base_mut().add_to_group("civilian");
            self.base_mut().emit_signal("student_allied", &[]);
            godot_print!("Student: 'I'm with you. Let's fight for real change.'");
        }
    }

    #[func]
    pub fn on_civilian_killed(&mut self) {
        if self.alignment != StudentAlignment::Neutral {
            return;
        }
        if self.trust == 0 {
            godot_print!("Student witnessed a killing — radicalized from trauma.");
            self.become_hostile();
        } else {
            godot_print!("Student is shaken but holds.");
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
        self.health = health.clamp(0, MAX_HP);
    }

    #[func]
    pub fn get_trust(&self) -> i32 {
        self.trust
    }

    #[func]
    pub fn set_trust(&mut self, trust: i32) {
        self.trust = trust.clamp(0, MAX_TRUST);
    }

    fn process_neutral(&mut self, delta: f64) {
        self.radicalize_check_timer += delta;
        if self.radicalize_check_timer >= 1.0 {
            self.radicalize_check_timer = 0.0;
            self.check_troll_influence();
        }

        if let Some(threat) = self.nearest_threat_position() {
            self.flee_from(threat);
        }

        if self.mob_state == MobState::Fleeing {
            self.flee_timer += delta;
            if self.flee_timer >= FLEE_DURATION {
                self.flee_timer = 0.0;
                self.flee_target = None;
                self.mob_state = MobState::Idle;
            } else if let Some(fp) = self.flee_target {
                self.move_toward(fp, FLEE_SPEED);
                return;
            }
        }

        self.wander_timer += delta;
        if self.wander_timer >= self.wander_interval {
            self.wander_timer = 0.0;
            self.pick_wander_target();
        }
        let target = self.wander_target;
        self.move_toward(target, WANDER_SPEED);
    }

    fn process_allied(&mut self, delta: f64) {
        self.rally_timer += delta;
        if self.rally_timer >= RALLY_INTERVAL {
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
            self.move_toward(player_pos, WANDER_SPEED * 1.2);
        } else {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
        }
    }

    fn process_radicalized(&mut self, delta: f64) {
        if !self.can_slash {
            self.slash_timer += delta;
            if self.slash_timer >= ATTACK_COOLDOWN {
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
        self.chase(player_pos, WANDER_SPEED * 1.5);

        if dist <= 32.0 && self.can_slash {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                player.bind_mut().take_damage(RADICALIZED_ATTACK);
                godot_print!("Radicalized Student attacks for {}!", RADICALIZED_ATTACK);
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

        if my_pos.distance_to(player_pos) <= RALLY_RADIUS {
            self.base_mut().emit_signal(
                "rally_triggered",
                &[
                    Variant::from(RALLY_SPEED_BONUS),
                    Variant::from(RALLY_DURATION),
                ],
            );
            godot_print!(
                "Student rallies! Player speed +{} for {}s.",
                RALLY_SPEED_BONUS,
                RALLY_DURATION
            );
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
                if my_pos.distance_to(body.get_global_position()) <= RADICALIZE_RANGE {
                    godot_print!("Troll influenced the Student.");
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
                if dist <= FEAR_RADIUS {
                    if nearest.map_or(true, |(d, _)| dist < d) {
                        nearest = Some((dist, epos));
                    }
                }
            }
        }
        nearest.map(|(_, pos)| pos)
    }

    fn pick_wander_target(&mut self) {
        let pos = self.base_mut().get_global_position();
        let offset = Vector2::new(
            (pseudo_rand() - 0.5) * WANDER_RADIUS * 2.0,
            (pseudo_rand() - 0.5) * WANDER_RADIUS * 2.0,
        );
        self.wander_target = pos + offset;
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
}

fn pseudo_rand() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (t % 10_000) as f32 / 10_000.0
}
