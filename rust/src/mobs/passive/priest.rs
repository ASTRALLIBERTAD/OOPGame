use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, MobState, PassiveBehavior};
use crate::rustplayer::Rustplayer;

const MAX_HP: i32 = 25;
const WANDER_SPEED: f32 = 30.0;
const FLEE_SPEED: f32 = 70.0;
const FLEE_DURATION: f64 = 4.0;
const FEAR_RADIUS: f32 = 160.0;
const HEAL_AMOUNT: i32 = 5;
const HEAL_COOLDOWN: f64 = 8.0;
const HEAL_RADIUS: f32 = 80.0;
const HEAL_THRESHOLD: i32 = 14;
const BLESSING_DURATION: f64 = 60.0;
const MAX_BLESSINGS: i32 = 3;
const WANDER_RADIUS: f32 = 120.0;
const SANCTUARY_RADIUS: f32 = 90.0;
const SANCTUARY_TICK: f64 = 1.0;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Priest {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    #[var(get = get_blessings_remaining)]
    blessings_remaining: i32,

    mob_state: MobState,

    wander_target: Vector2,
    wander_timer: f64,
    wander_interval: f64,

    flee_target: Option<Vector2>,
    flee_timer: f64,

    heal_timer: f64,
    sanctuary_timer: f64,
}

#[godot_api]
impl ICharacterBody2D for Priest {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: MAX_HP,
            blessings_remaining: MAX_BLESSINGS,
            mob_state: MobState::Idle,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            wander_interval: 6.0,
            flee_target: None,
            flee_timer: 0.0,
            heal_timer: 0.0,
            sanctuary_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("neutral");
        self.base_mut().add_to_group("civilian");
        self.base_mut().add_to_group("healer");
        let pos = self.base_mut().get_global_position();
        self.wander_target = pos;
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.tick_auto_heal(delta);
        self.tick_sanctuary(delta);

        if let Some(threat) = self.nearest_threat_position() {
            self.flee(threat);
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
            self.wander();
        }
        let target = self.wander_target;
        self.move_toward(target, WANDER_SPEED);
    }
}

impl Entity for Priest {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.base_mut().emit_signal("priest_killed", &[]);
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

impl PassiveBehavior for Priest {
    fn wander(&mut self) {
        let pos = self.base_mut().get_global_position();
        let offset = Vector2::new(
            (pseudo_rand() - 0.5) * WANDER_RADIUS * 2.0,
            (pseudo_rand() - 0.5) * WANDER_RADIUS * 2.0,
        );
        self.wander_target = pos + offset;
    }

    fn flee(&mut self, from: Vector2) {
        let pos = self.base_mut().get_global_position();
        let away = (pos - from).normalized();
        self.flee_target = Some(pos + away * 200.0);
        self.flee_timer = 0.0;
        self.mob_state = MobState::Fleeing;
        godot_print!("Priest flees from danger.");
    }
}

#[godot_api]
impl Priest {
    #[signal]
    fn blessing_granted(duration: f64);

    #[signal]
    fn heal_player(amount: i32);

    #[signal]
    fn sanctuary_pulse(position: Vector2);

    #[signal]
    fn priest_killed();

    #[func]
    pub fn on_interact(&mut self) {
        let tree = self.base_mut().get_tree();
        let players = tree.get_nodes_in_group("player");

        for node in players.iter_shared() {
            if let Ok(mut player_gd) = node.try_cast::<Rustplayer>() {
                if self.blessings_remaining > 0 {
                    self.blessings_remaining -= 1;
                    let heal = HEAL_AMOUNT * 2;
                    player_gd.bind_mut().heal(heal);
                    self.base_mut()
                        .emit_signal("blessing_granted", &[Variant::from(BLESSING_DURATION)]);
                    self.base_mut()
                        .emit_signal("heal_player", &[Variant::from(heal)]);
                    godot_print!(
                        "Priest blesses the player. Blessings remaining: {}",
                        self.blessings_remaining
                    );
                } else {
                    let heal = HEAL_AMOUNT / 2;
                    player_gd.bind_mut().heal(heal);
                    self.base_mut()
                        .emit_signal("heal_player", &[Variant::from(heal)]);
                    godot_print!("Priest: 'I have given all I can. Stay safe.'");
                }
                break;
            }
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
    pub fn get_blessings_remaining(&self) -> i32 {
        self.blessings_remaining
    }

    fn tick_sanctuary(&mut self, delta: f64) {
        self.sanctuary_timer += delta;
        if self.sanctuary_timer < SANCTUARY_TICK {
            return;
        }
        self.sanctuary_timer = 0.0;

        let my_pos = self.base_mut().get_global_position();
        let tree = self.base_mut().get_tree();
        let players = tree.get_nodes_in_group("player");

        for node in players.iter_shared() {
            if let Ok(player_gd) = node.try_cast::<Rustplayer>() {
                let player_pos = player_gd.get_global_position();
                if my_pos.distance_to(player_pos) <= SANCTUARY_RADIUS {
                    self.base_mut()
                        .emit_signal("sanctuary_pulse", &[Variant::from(my_pos)]);
                    godot_print!("Priest: Sanctuary active — player protected.");
                    break;
                }
            }
        }
    }

    fn tick_auto_heal(&mut self, delta: f64) {
        self.heal_timer += delta;
        if self.heal_timer < HEAL_COOLDOWN {
            return;
        }

        let my_pos = self.base_mut().get_global_position();
        let tree = self.base_mut().get_tree();
        let players = tree.get_nodes_in_group("player");

        for node in players.iter_shared() {
            if let Ok(mut player_gd) = node.try_cast::<Rustplayer>() {
                let player_pos = player_gd.get_global_position();
                let hp = player_gd.bind().get_health();
                if my_pos.distance_to(player_pos) <= HEAL_RADIUS && hp <= HEAL_THRESHOLD {
                    self.heal_timer = 0.0;
                    player_gd.bind_mut().heal(HEAL_AMOUNT);
                    self.base_mut()
                        .emit_signal("heal_player", &[Variant::from(HEAL_AMOUNT)]);
                    godot_print!("Priest healed the player for {}.", HEAL_AMOUNT);
                    break;
                }
            }
        }
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
