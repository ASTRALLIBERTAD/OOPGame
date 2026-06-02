use godot::classes::{AnimatedSprite2D, Area2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;

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
    sanctuary_area: OnEditor<Gd<Area2D>>,

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

    home_position: Vector2,

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
            sanctuary_area: OnEditor::default(),
            health: MAX_HP,
            blessings_remaining: MAX_BLESSINGS,
            mob_state: MobState::Idle,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            wander_interval: 6.0,
            home_position: Vector2::ZERO,
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
        self.home_position = pos;
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
        let angle = (godot::global::randf() * std::f64::consts::TAU) as f32;
        let dist = godot::global::randf() as f32 * WANDER_RADIUS;
        let offset = Vector2::new(angle.cos() * dist, angle.sin() * dist);
        self.wander_target = self.home_position + offset;
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
            let Ok(body) = node.try_cast::<CharacterBody2D>() else {
                continue;
            };
            let Ok(mut player_gd) = body.try_cast::<Rustplayer>() else {
                continue;
            };
            if self.blessings_remaining > 0 {
                self.blessings_remaining -= 1;
                let heal = HEAL_AMOUNT * 2;
                player_gd.bind_mut().heal(heal);
                // player_gd.bind_mut().apply_blessing(BLESSING_DURATION);
                let mut event_bus = get_autoload_by_name::<Node>("EventBus");
                event_bus.call(
                    "emit_signal",
                    &[
                        Variant::from(GString::from("message")),
                        Variant::from(GString::from(
                            format!(
                                "Priest blesses you. +{} HP. Blessings left: {}",
                                heal, self.blessings_remaining
                            )
                            .as_str(),
                        )),
                    ],
                );
            } else {
                let heal = HEAL_AMOUNT / 2;
                player_gd.bind_mut().heal(heal);
                let mut event_bus = get_autoload_by_name::<Node>("EventBus");
                event_bus.call(
                    "emit_signal",
                    &[
                        Variant::from(GString::from("message")),
                        Variant::from(GString::from(
                            "Priest: 'I have given all I can. Stay safe.'",
                        )),
                    ],
                );
            }
            break;
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
        let bodies = self.sanctuary_area.get_overlapping_bodies();

        for body in bodies.iter_shared() {
            if body.clone().is_in_group("enemy") {
                if let Ok(mut enemy) = body.clone().try_cast::<CharacterBody2D>() {
                    enemy.set_velocity(Vector2::ZERO);
                    enemy.move_and_slide();
                }
            }

            if body.is_in_group("player") {
                if let Ok(body2) = body.try_cast::<CharacterBody2D>() {
                    if let Ok(mut player) = body2.try_cast::<Rustplayer>() {
                        player.bind_mut().set_in_sanctuary(true);
                    }
                }
            }
        }

        let players = self.base_mut().get_tree().get_nodes_in_group("player");
        for node in players.iter_shared() {
            if let Ok(body) = node.try_cast::<CharacterBody2D>() {
                let dist = my_pos.distance_to(body.get_global_position());
                if dist > SANCTUARY_RADIUS {
                    if let Ok(mut player) = body.try_cast::<Rustplayer>() {
                        player.bind_mut().set_in_sanctuary(false);
                    }
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
        let bosses = self.base_mut().get_tree().get_nodes_in_group("boss");
        let mut nearest: Option<(f32, Vector2)> = None;
        for boss in bosses.iter_shared() {
            if let Ok(body) = boss.try_cast::<CharacterBody2D>() {
                let epos = body.get_global_position();
                let dist = my_pos.distance_to(epos);
                if dist <= FEAR_RADIUS && nearest.is_none_or(|(d, _)| dist < d) {
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
}
