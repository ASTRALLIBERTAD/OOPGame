use godot::classes::{AnimatedSprite2D, Area2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, HostileBehavior, MobState};
use crate::rustplayer::Rustplayer;

const MAX_HP: i32 = 60;
const INFLUENCE_RADIUS: f32 = 400.0;
const INFLUENCE_DAMAGE_BONUS: i32 = 5;
const ARREST_DURATION: f64 = 3.0;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct OrderForce {
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

    mob_state: MobState,
    can_slash: bool,
    slash_timer: f64,

    boss_nearby: bool,
    arrest_timer: f64,
    arresting: bool,
}

#[godot_api]
impl ICharacterBody2D for OrderForce {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            attack_area: OnEditor::default(),
            health: MAX_HP,
            speed: 70.0,
            aggro_range: 250.0,
            attack_damage: 8,
            attack_cooldown: 2.0,
            mob_state: MobState::Idle,
            can_slash: true,
            slash_timer: 0.0,
            boss_nearby: false,
            arrest_timer: 0.0,
            arresting: false,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("enemy");
        self.base_mut().add_to_group("order_force");
        self.attack_area.set_monitoring(true);
        self.attack_area.set_monitorable(false);
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.tick_attack_cooldown(delta);
        self.tick_arrest(delta);
        self.check_boss_influence();

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

        if self.arresting {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
            return;
        }

        self.chase(player_pos, self.speed);

        if distance <= 40.0 && self.can_slash {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                let dmg = self.effective_damage();
                player.bind_mut().take_damage(dmg);
                godot_print!("Puersa ng Orden strikes for {} damage!", dmg);

                if self.boss_nearby && !player.bind().is_arrested() {
                    player.bind_mut().apply_arrested(ARREST_DURATION);
                    self.arresting = true;
                    self.arrest_timer = 0.0;
                    godot_print!("Puersa ng Orden: 'You are under arrest!'");
                }
            }
            self.can_slash = false;
            self.slash_timer = 0.0;
        }
    }
}

impl Entity for OrderForce {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(MAX_HP);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl HostileBehavior for OrderForce {
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
        target.take_damage(self.effective_damage());
    }
}

#[godot_api]
impl OrderForce {
    fn effective_damage(&self) -> i32 {
        if self.boss_nearby {
            self.attack_damage + INFLUENCE_DAMAGE_BONUS
        } else {
            self.attack_damage
        }
    }

    fn tick_attack_cooldown(&mut self, delta: f64) {
        if !self.can_slash {
            self.slash_timer += delta;
            if self.slash_timer >= self.attack_cooldown {
                self.can_slash = true;
                self.slash_timer = 0.0;
            }
        }
    }

    fn tick_arrest(&mut self, delta: f64) {
        if !self.arresting {
            return;
        }
        self.arrest_timer += delta;
        if self.arrest_timer >= ARREST_DURATION {
            self.arresting = false;
            self.arrest_timer = 0.0;
        }
    }

    fn check_boss_influence(&mut self) {
        let my_pos = self.base_mut().get_global_position();
        let boss_nodes = self.base_mut().get_tree().get_nodes_in_group("boss");

        let nearby = boss_nodes.iter_shared().any(|node| {
            node.try_cast::<CharacterBody2D>()
                .map(|boss| my_pos.distance_to(boss.get_global_position()) <= INFLUENCE_RADIUS)
                .unwrap_or(false)
        });

        self.boss_nearby = nearby;
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, MAX_HP);
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }
}
