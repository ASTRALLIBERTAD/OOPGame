use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;
use rand::RngExt;

use crate::entity::{Entity, HostileBehavior, MobState, NeutralBehavior};
use crate::rustplayer::Rustplayer;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct RoamingTrader {
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
    wander_radius: f32,

    #[export]
    wander_interval: f64,

    #[export]
    markup: f32,

    #[export]
    aggro_range: f32,

    #[export]
    attack_damage: i32,

    #[export]
    attack_cooldown: f64,

    mob_state: MobState,
    is_hostile: bool,
    home_position: Vector2,
    wander_target: Vector2,
    wander_timer: f64,
    can_slash: bool,
    slash_timer: f64,

    in_trade: bool,

    playing_oneshot: bool,
    flash_timer: f64,
}

#[godot_api]
impl ICharacterBody2D for RoamingTrader {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: 40,
            wander_speed: 50.0,
            wander_radius: 300.0,
            wander_interval: 4.0,
            markup: 1.2,
            aggro_range: 160.0,
            attack_damage: 5,
            attack_cooldown: 1.2,
            mob_state: MobState::Idle,
            is_hostile: false,
            home_position: Vector2::ZERO,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            can_slash: true,
            slash_timer: 0.0,
            in_trade: false,

            playing_oneshot: false,
            flash_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("neutral");
        self.base_mut().add_to_group("trader");
        let pos = self.base_mut().get_global_position();
        self.wander_target = pos;
        self.home_position = pos;
        let callable = self.base().callable("on_animation_finished");
        self.sprite.connect("animation_finished", &callable);
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        if self.flash_timer > 0.0 {
            self.flash_timer -= delta;
            if self.flash_timer <= 0.0 {
                self.flash_timer = 0.0;
                self.base_mut().set_modulate(Color::WHITE);
            }
        }

        if self.playing_oneshot {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
            return;
        }

        if self.is_hostile {
            self.process_hostile(delta);
        } else {
            self.process_neutral(delta);
        }
    }
}

impl Entity for RoamingTrader {
    fn take_damage(&mut self, amount: i32) {
        if !self.is_alive() {
            return;
        }
        self.health = (self.health - amount).max(0);
        self.base_mut().set_modulate(Color::from_rgb(1.0, 0.3, 0.3));
        self.flash_timer = 0.2;
        if !self.is_hostile {
            self.become_hostile();
        }
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.playing_oneshot = true;
            self.sprite.play_ex().name("death").done();
            let mut rng = rand::rng();
            let multiplier: f32 = rng.random_range(0.5..=1.5);
            let drop = ((self.health as f32 * multiplier) as i32 + 20).max(10);
            let pos = self.base_mut().get_global_position();
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("piso_dropped")),
                    Variant::from(drop),
                    Variant::from(pos),
                ],
            );
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from("Tagapamagitan has been killed.")),
                ],
            );
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(40);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl NeutralBehavior for RoamingTrader {
    fn interact(&self) -> &'static str {
        "dialogue.tagapamagitan.greet"
    }

    fn become_hostile(&mut self) {
        if self.is_hostile {
            return;
        }
        self.is_hostile = true;
        self.base_mut().remove_from_group("neutral");
        self.base_mut().add_to_group("enemy");
        self.markup = 2.5;
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from(
                    "Tagapamagitan: 'If that is how you want it, I can fight too!'",
                )),
            ],
        );
    }
}

impl HostileBehavior for RoamingTrader {
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
impl RoamingTrader {
    #[signal]
    fn turned_hostile();

    #[func]
    fn set_in_trade(&mut self, value: bool) {
        self.in_trade = value;
    }

    fn process_neutral(&mut self, delta: f64) {
        if self.in_trade {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
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

    fn process_hostile(&mut self, delta: f64) {
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
        let distance = my_pos.distance_to(player_pos);

        if distance <= self.aggro_range {
            self.aggro(player_pos);
            self.chase(player_pos, self.wander_speed * 1.4);
            if distance <= 40.0 && self.can_slash {
                if let Ok(mut p) = player_gd.try_cast::<Rustplayer>() {
                    let dmg = self.attack_damage;
                    p.bind_mut().take_damage(dmg);
                }
                self.can_slash = false;
                self.slash_timer = 0.0;
            }
        } else {
            self.mob_state = MobState::Idle;
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
        }
    }

    fn pick_wander_target(&mut self) {
        let angle = (godot::global::randf() * std::f64::consts::TAU) as f32;
        let dist = godot::global::randf() as f32 * self.wander_radius;
        let offset = Vector2::new(angle.cos() * dist, angle.sin() * dist);
        self.wander_target = self.home_position + offset;
    }

    fn move_toward(&mut self, target: Vector2, speed: f32) {
        let pos = self.base_mut().get_global_position();
        if pos.distance_to(target) < 8.0 {
            if speed == self.wander_speed {
                self.wander_timer = self.wander_interval;
            }
            self.base_mut().set_velocity(Vector2::ZERO);
            if self.sprite.get_animation().to_string() != "default" {
                self.sprite.play_ex().name("default").done();
            }
        } else {
            let dir = (target - pos).normalized();
            self.sprite.set_flip_h(dir.x < 0.0);
            self.base_mut().set_velocity(dir * speed);
            if self.sprite.get_animation().to_string() != "walking_running" {
                self.sprite.play_ex().name("walking_running").done();
            }
        }
        self.base_mut().move_and_slide();
    }

    #[func]
    pub fn on_interact(&mut self) {
        if self.is_hostile {
            return;
        }
        self.playing_oneshot = true;
        self.sprite.play_ex().name("interact").done();
        let markup = self.markup;
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("trade_requested")),
                Variant::from(markup),
                Variant::from(self.base().clone().upcast::<Node>()),
            ],
        );
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from(
                    format!(
                        "Tagapamagitan: 'Welcome! Markup: {:.0}%'",
                        (markup - 1.0) * 100.0
                    )
                    .as_str(),
                )),
            ],
        );
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, 40);
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }

    #[func]
    fn on_animation_finished(&mut self) {
        self.playing_oneshot = false;
        if self.mob_state == MobState::Dead {
            self.base_mut().queue_free();
        } else {
            self.sprite.play_ex().name("default").done();
        }
    }
}
