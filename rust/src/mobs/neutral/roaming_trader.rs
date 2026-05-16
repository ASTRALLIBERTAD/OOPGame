use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

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
    markup: f32,

    mob_state: MobState,
    is_hostile: bool,

    wander_target: Vector2,
    wander_timer: f64,
    wander_interval: f64,

    attack_damage: i32,
    attack_cooldown: f64,
    can_slash: bool,
    slash_timer: f64,
    aggro_range: f32,
}

#[godot_api]
impl ICharacterBody2D for RoamingTrader {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: 40,
            wander_speed: 50.0,
            markup: 1.2,
            mob_state: MobState::Idle,
            is_hostile: false,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            wander_interval: 4.0,
            attack_damage: 5,
            attack_cooldown: 1.2,
            can_slash: true,
            slash_timer: 0.0,
            aggro_range: 160.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("neutral");
        self.base_mut().add_to_group("trader");
        let global_position = self.base_mut().get_global_position();
        self.wander_target = global_position;
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
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

        if !self.is_hostile {
            self.become_hostile();
        }

        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.base_mut().emit_signal("trader_killed", &[]);
            self.base_mut().queue_free();
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
        godot_print!("Tagapamagitan: 'If that is how you want it, I can fight too!'");
        self.base_mut().emit_signal("turned_hostile", &[]);
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
    fn trade_requested();

    #[signal]
    fn turned_hostile();

    #[signal]
    fn trader_killed();

    fn process_neutral(&mut self, delta: f64) {
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
        let pos = self.base_mut().get_global_position();
        let offset = Vector2::new((pseudo_rand() - 0.5) * 300.0, (pseudo_rand() - 0.5) * 300.0);
        self.wander_target = pos + offset;
    }

    fn move_toward(&mut self, target: Vector2, speed: f32) {
        let pos = self.base_mut().get_global_position();
        if pos.distance_to(target) < 8.0 {
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
        if self.is_hostile {
            return;
        }
        self.base_mut().emit_signal("trade_requested", &[]);
        godot_print!(
            "Tagapamagitan: '{}' (markup: {:.0}%)",
            self.interact(),
            (self.markup - 1.0) * 100.0
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
}

fn pseudo_rand() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (t % 10_000) as f32 / 10_000.0
}
