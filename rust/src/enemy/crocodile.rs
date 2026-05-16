use godot::classes::{AnimatedSprite2D, Area2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, HostileBehavior, MobState};
use crate::rustplayer::Rustplayer;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Crocodile {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    attack_area: OnEditor<Gd<Area2D>>,

    #[export]
    speed: f32,

    #[export]
    aggro_range: f32,

    #[export]
    attack_damage: i32,

    #[export]
    attack_cooldown: f64,

    mob_state: MobState,
    can_slash: bool,
    slash_timer: f64,
}

#[godot_api]
impl ICharacterBody2D for Crocodile {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: 100,
            attack_area: OnEditor::default(),
            speed: 80.0,
            aggro_range: 200.0,
            attack_damage: 10,
            attack_cooldown: 0.8,
            mob_state: MobState::Idle,
            can_slash: true,
            slash_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("enemy");
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        if !self.can_slash {
            self.slash_timer += delta;
            if self.slash_timer >= self.attack_cooldown {
                self.can_slash = true;
                self.slash_timer = 0.0;
            }
        }

        let my_pos = self.base_mut().get_global_position();

        let player_node = self
            .base_mut()
            .get_tree()
            .get_nodes_in_group("player")
            .get(0);

        let Some(player_node) = player_node else {
            return;
        };

        let Ok(player_gd) = player_node.try_cast::<CharacterBody2D>() else {
            return;
        };

        let player_pos = player_gd.get_global_position();
        let distance = my_pos.distance_to(player_pos);

        if distance <= self.aggro_range {
            self.aggro(player_pos);
            self.chase(player_pos, self.speed);

            if distance <= 40.0 && self.can_slash {
                if let Ok(mut rustplayer) = player_gd.try_cast::<Rustplayer>() {
                    let damage = self.attack_damage;
                    rustplayer.bind_mut().take_damage(damage);
                    godot_print!("Crocodile attacked player for {} damage!", damage);
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
}

impl Entity for Crocodile {
    fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health += amount;
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl HostileBehavior for Crocodile {
    fn aggro(&mut self, target: Vector2) {
        self.mob_state = MobState::Aggro;
        let _ = target;
    }

    fn chase(&mut self, target: Vector2, speed: f32) {
        let current_pos = self.base_mut().get_global_position();
        let direction = (target - current_pos).normalized();
        self.sprite.set_flip_h(direction.x < 0.0);
        self.base_mut().set_velocity(direction * speed);
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
impl Crocodile {
    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health;
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }
}
