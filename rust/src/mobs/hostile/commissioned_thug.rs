use godot::classes::{AnimatedSprite2D, Area2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, HostileBehavior, MobState};
use crate::rustplayer::Rustplayer;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct CommissionedThug {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    attack_area: OnEditor<Gd<Area2D>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    speed: f32,

    #[export]
    aggro_range: f32,

    #[export]
    attack_damage: i32,

    #[export]
    attack_cooldown: f64,

    #[export]
    #[var(get = get_corruption_level, set = set_corruption_level)]
    corruption_level: i32,

    #[export]
    toll_amount: i32,

    mob_state: MobState,
    toll_demanded: bool,
    toll_cooldown: f64,
    toll_timer: f64,
    can_slash: bool,
    slash_timer: f64,
}

#[godot_api]
impl ICharacterBody2D for CommissionedThug {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            attack_area: OnEditor::default(),
            health: 60,
            speed: 85.0,
            aggro_range: 150.0,
            attack_damage: 8,
            attack_cooldown: 1.0,
            corruption_level: 0,
            toll_amount: 100,
            mob_state: MobState::Idle,
            toll_demanded: false,
            toll_cooldown: 6.0,
            toll_timer: 0.0,
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

        if self.corruption_level < 5 && !self.toll_demanded && self.toll_amount > 0 {
            self.tick_toll_demand(delta);
            return;
        }

        self.aggro(player_pos);
        self.chase(player_pos, self.speed);

        if distance <= 45.0 && self.can_slash {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                let dmg = self.attack_damage;
                player.bind_mut().take_damage(dmg);
                godot_print!("Komisyon Goon hits player for {}!", dmg);
            }
            self.can_slash = false;
            self.slash_timer = 0.0;
        }
    }
}

impl Entity for CommissionedThug {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(60);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl HostileBehavior for CommissionedThug {
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
impl CommissionedThug {
    #[signal]
    fn toll_demanded(amount: i32);

    fn tick_toll_demand(&mut self, delta: f64) {
        self.toll_timer += delta;
        if self.toll_timer >= self.toll_cooldown {
            self.toll_timer = 0.0;
            let toll_amount = self.toll_amount;
            self.base_mut()
                .emit_signal("toll_demanded", &[Variant::from(toll_amount)]);
            godot_print!(
                "Komisyon Goon: 'Pay {} piso before you pass.'",
                self.toll_amount
            );
            self.toll_demanded = true;
        }
    }

    #[func]
    pub fn on_toll_paid(&mut self) {
        godot_print!("Komisyon Goon: 'Move along.'");
        self.mob_state = MobState::Idle;
        self.base_mut().set_velocity(Vector2::ZERO);
    }

    #[func]
    pub fn on_toll_refused(&mut self) {
        self.toll_demanded = true;
        self.attack_damage += 2;
        godot_print!("Komisyon Goon: 'Your loss.'");
    }

    #[func]
    pub fn set_corruption_level(&mut self, level: i32) {
        self.corruption_level = level.clamp(0, 10);
        if self.corruption_level >= 5 {
            self.toll_demanded = true;
            self.attack_damage = (self.attack_damage as f32 * 1.3) as i32;
        }
    }

    #[func]
    pub fn get_corruption_level(&self) -> i32 {
        self.corruption_level
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, 60);
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }
}
