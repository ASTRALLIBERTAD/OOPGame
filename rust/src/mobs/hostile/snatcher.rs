use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, HostileBehavior, MobState};
use crate::rustplayer::Rustplayer;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Snatcher {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    speed: f32,

    #[export]
    aggro_range: f32,

    #[export]
    steal_amount: i32,

    #[export]
    steal_range: f32,

    mob_state: MobState,
    has_stolen: bool,
    stole_piso_successfully: bool,
    flee_target: Option<Vector2>,
}

#[godot_api]
impl ICharacterBody2D for Snatcher {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: 15,
            speed: 50.0,
            aggro_range: 120.0,
            steal_amount: 50,
            steal_range: 20.0,
            mob_state: MobState::Idle,
            has_stolen: false,
            stole_piso_successfully: false,
            flee_target: None,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("enemy");
    }

    fn process(&mut self, _delta: f64) {
        if !self.is_alive() {
            return;
        }

        if self.has_stolen {
            self.mob_state = MobState::Fleeing;
            if let Some(flee_pos) = self.flee_target {
                let pos = self.base_mut().get_global_position();
                let dir = (flee_pos - pos).normalized();
                self.sprite.set_flip_h(dir.x < 0.0);
                let speed = self.speed;
                self.base_mut().set_velocity(dir * speed * 1.5);
                self.base_mut().move_and_slide();
                if pos.distance_to(flee_pos) < 20.0 {
                    self.base_mut().queue_free();
                }
            }
            return;
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

        if distance > self.aggro_range {
            self.mob_state = MobState::Idle;
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
            return;
        }

        self.aggro(player_pos);
        self.chase(player_pos, self.speed);

        if distance <= self.steal_range {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                self.try_steal(&mut player.bind_mut());
            }
        }
    }
}

impl Entity for Snatcher {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if !self.is_alive() {
            if self.has_stolen {
                let steal_amount = self.steal_amount;
                self.base_mut()
                    .emit_signal("drop_stolen_piso", &[Variant::from(steal_amount)]);
            }
            self.mob_state = MobState::Dead;
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(15);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl HostileBehavior for Snatcher {
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

    fn attack(&mut self, _target: &mut dyn Entity) {}
}

#[godot_api]
impl Snatcher {
    #[signal]
    fn piso_stolen(amount: i32);

    #[signal]
    fn drop_stolen_piso(amount: i32);

    fn try_steal(&mut self, player: &mut godot::obj::GdMut<Rustplayer>) {
        if self.has_stolen {
            return;
        }

        let steal_amount = self.steal_amount;
        let had_piso = player.spend_piso(steal_amount);
        self.stole_piso_successfully = had_piso;

        let msg = if had_piso {
            self.base_mut()
                .emit_signal("piso_stolen", &[Variant::from(steal_amount)]);
            format!("Snatcher stole {} piso!", steal_amount)
        } else {
            "Snatcher tried to steal but player had no piso.".to_string()
        };

        player
            .base_mut()
            .emit_signal("message", &[Variant::from(msg.clone())]);

        godot_print!("{}", msg);

        self.has_stolen = true;

        let my_pos = self.base_mut().get_global_position();
        let player_pos = player.base().get_global_position();
        let away = (my_pos - player_pos).normalized();
        self.flee_target = Some(my_pos + away * 600.0);
    }
    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, 15);
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }
}
