use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;
use rand::RngExt;

use crate::entity::{Entity, HostileBehavior, MobState};
use crate::rustplayer::Rustplayer;

const MAX_HP: i32 = 45;
const BRIBE_POOL_MAX: i32 = 500;
const MOB_BUFF_RADIUS: f32 = 220.0;
const MOB_BUFF_COOLDOWN: f64 = 8.0;
const MOB_BUFF_COST_PER_TARGET: i32 = 50;
const PLAYER_BRIBE_COOLDOWN: f64 = 15.0;
const FLEE_HP_THRESHOLD: i32 = 15;
const FLEE_SPEED: f32 = 115.0;
const FLEE_DISTANCE: f32 = 420.0;

/// Trapo Fixer — pays off nearby mobs, bribes the player, and cuts and runs when threatened.
/// Drops remaining black funds on death or escape.
#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct CorruptionBroker {
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
    attack_damage: i32,

    #[export]
    attack_cooldown: f64,

    #[export]
    #[var(get = get_bribe_pool, set = set_bribe_pool)]
    bribe_pool: i32,

    #[export]
    #[var(get = get_corruption_level, set = set_corruption_level)]
    corruption_level: i32,

    mob_state: MobState,

    can_slash: bool,
    slash_timer: f64,

    player_bribe_timer: f64,
    mob_buff_timer: f64,

    player_deal_resolved: bool,

    flee_target: Option<Vector2>,
}

#[godot_api]
impl ICharacterBody2D for CorruptionBroker {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: MAX_HP,
            speed: 70.0,
            aggro_range: 200.0,
            attack_damage: 6,
            attack_cooldown: 1.8,
            bribe_pool: BRIBE_POOL_MAX,
            corruption_level: 0,
            mob_state: MobState::Idle,
            can_slash: true,
            slash_timer: 0.0,
            player_bribe_timer: PLAYER_BRIBE_COOLDOWN,
            mob_buff_timer: 0.0,
            player_deal_resolved: false,
            flee_target: None,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("enemy");
        self.base_mut().add_to_group("corruption_broker");
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.tick_attack_cooldown(delta);
        self.tick_mob_buff(delta);

        if self.health <= FLEE_HP_THRESHOLD {
            self.process_flee(delta);
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

        if !self.player_deal_resolved && self.bribe_pool >= 80 {
            self.player_bribe_timer += delta;
            if self.player_bribe_timer >= PLAYER_BRIBE_COOLDOWN {
                self.player_bribe_timer = 0.0;
                let offer = ((self.corruption_level + 1) * 80).clamp(80, self.bribe_pool);
                let mut event_bus = get_autoload_by_name::<Node>("EventBus");
                let self_node = self.base().clone().upcast::<Node>();
                event_bus.call(
                    "emit_signal",
                    &[
                        Variant::from(GString::from("bribe_requested")),
                        Variant::from(GString::from("broker")),
                        Variant::from(offer),
                        Variant::from(self_node),
                    ],
                );
            }
        }
        self.aggro(player_pos);
        self.chase(player_pos, self.speed);

        if distance <= 40.0 && self.can_slash {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                let dmg = self.attack_damage;
                player.bind_mut().take_damage(dmg);
                godot_print!("Trapo Fixer strikes for {}!", dmg);
            }
            self.can_slash = false;
            self.slash_timer = 0.0;
        }
    }
}

impl Entity for CorruptionBroker {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.on_death();
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

impl HostileBehavior for CorruptionBroker {
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
impl CorruptionBroker {
    fn tick_attack_cooldown(&mut self, delta: f64) {
        if !self.can_slash {
            self.slash_timer += delta;
            if self.slash_timer >= self.attack_cooldown {
                self.can_slash = true;
                self.slash_timer = 0.0;
            }
        }
    }

    fn tick_mob_buff(&mut self, delta: f64) {
        if self.bribe_pool <= 0 {
            return;
        }

        self.mob_buff_timer += delta;
        if self.mob_buff_timer < MOB_BUFF_COOLDOWN {
            return;
        }
        self.mob_buff_timer = 0.0;

        let my_pos = self.base_mut().get_global_position();
        let my_id = self.base_mut().instance_id();
        let enemies = self.base_mut().get_tree().get_nodes_in_group("enemy");

        let mut targets: Vec<Gd<CharacterBody2D>> = Vec::new();
        for node in enemies.iter_shared() {
            if node.instance_id() == my_id {
                continue;
            }
            if let Ok(body) = node.try_cast::<CharacterBody2D>() {
                if my_pos.distance_to(body.get_global_position()) <= MOB_BUFF_RADIUS {
                    targets.push(body);
                }
            }
        }

        let count = targets.len() as i32;
        if count == 0 {
            return;
        }

        let total_cost = (MOB_BUFF_COST_PER_TARGET * count).min(self.bribe_pool);
        self.bribe_pool -= total_cost;

        self.base_mut()
            .emit_signal("mob_buffed", &[Variant::from(my_pos), Variant::from(count)]);
        godot_print!(
            "Trapo Fixer pays off {} mobs ({} piso). Pool: {}",
            count,
            total_cost,
            self.bribe_pool
        );
    }

    fn process_flee(&mut self, delta: f64) {
        let _ = delta;
        self.mob_state = MobState::Fleeing;

        let my_pos = self.base_mut().get_global_position();

        if self.flee_target.is_none() {
            let Some(player_node) = self
                .base_mut()
                .get_tree()
                .get_nodes_in_group("player")
                .get(0)
            else {
                return;
            };
            if let Ok(player_gd) = player_node.try_cast::<CharacterBody2D>() {
                let away = (my_pos - player_gd.get_global_position()).normalized();
                self.flee_target = Some(my_pos + away * FLEE_DISTANCE);
                godot_print!("Trapo Fixer: 'I'll be back with better leverage!'");
            }
        }

        if let Some(fp) = self.flee_target {
            let dir = (fp - my_pos).normalized();
            self.sprite.set_flip_h(dir.x < 0.0);
            self.base_mut().set_velocity(dir * FLEE_SPEED);
            self.base_mut().move_and_slide();

            if my_pos.distance_to(fp) < 20.0 {
                // Escaped — drop funds at exit point and despawn.
                let remaining = self.bribe_pool;
                let mut rng = rand::rng();

                let random_x = rng.random_range(-50.0..=50.0);
                let random_y = rng.random_range(-50.0..=50.0);

                let mut pos = self.base_mut().get_global_position();
                pos += Vector2::new(random_x, random_y);

                let mut event_bus = get_autoload_by_name::<Node>("EventBus");
                event_bus.call(
                    "emit_signal",
                    &[
                        Variant::from(GString::from("piso_dropped")),
                        Variant::from(remaining),
                        Variant::from(pos),
                    ],
                );
                godot_print!(
                    "Trapo Fixer escaped. {} piso in black funds left behind.",
                    remaining
                );
                self.base_mut().queue_free();
            }
        }
    }

    fn on_death(&mut self) {
        let mut rng = rand::rng();
        let base_pos = self.base_mut().get_global_position();
        let remaining = self.bribe_pool;
        let drops = rng.random_range(2..=4);
        let per_drop = (remaining / drops).max(1);
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");

        for _ in 0..drops {
            let random_x: f32 = rng.random_range(-60.0..=60.0);
            let random_y: f32 = rng.random_range(-60.0..=60.0);
            let pos = base_pos + Vector2::new(random_x, random_y);
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("piso_dropped")),
                    Variant::from(per_drop),
                    Variant::from(pos),
                ],
            );
        }
        godot_print!(
            "Trapo Fixer defeated. Black funds ({} piso) scattered in {} drops.",
            remaining,
            drops
        );
    }

    #[func]
    pub fn on_player_bribe_accepted(&mut self, amount: i32) {
        self.bribe_pool = (self.bribe_pool - amount).max(0);
        self.player_deal_resolved = true;
        // Caller is responsible for applying the Indebted debuff to the player.
        godot_print!("Trapo Fixer: 'Pleasure doing business. Remember who keeps order here.'");
    }

    #[func]
    pub fn on_player_bribe_rejected(&mut self) {
        self.player_deal_resolved = true;
        self.attack_damage = (self.attack_damage as f32 * 1.3) as i32;
        self.speed += 15.0;
        godot_print!("Trapo Fixer: 'You'll regret disrupting the arrangement.'");
    }

    #[func]
    pub fn set_corruption_level(&mut self, level: i32) {
        self.corruption_level = level.clamp(0, 10);
        if self.corruption_level >= 5 {
            // High-corruption environment inflates the war chest and sharpens elbows.
            self.bribe_pool = (self.bribe_pool as f32 * 1.5) as i32;
            self.attack_damage = (self.attack_damage as f32 * 1.2) as i32;
            godot_print!("Trapo Fixer: emboldened by high corruption. War chest expanded.");
        }
    }

    #[func]
    pub fn get_corruption_level(&self) -> i32 {
        self.corruption_level
    }

    #[func]
    pub fn set_bribe_pool(&mut self, amount: i32) {
        self.bribe_pool = amount.clamp(0, BRIBE_POOL_MAX);
    }

    #[func]
    pub fn get_bribe_pool(&self) -> i32 {
        self.bribe_pool
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
