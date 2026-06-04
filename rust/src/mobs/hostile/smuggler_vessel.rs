use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;

use crate::entity::{Entity, HostileBehavior, MobState};
use crate::rustplayer::Rustplayer;

const MAX_HP: i32 = 120;
const PATROL_SPEED: f32 = 40.0;
const CHASE_SPEED: f32 = 70.0;
const RAM_CHARGE_SPEED: f32 = 220.0;
const RAM_DAMAGE: i32 = 22;
const RAM_RANGE: f32 = 280.0;
const RAM_WINDUP: f64 = 1.2;
const RAM_COOLDOWN: f64 = 9.0;
const RAM_TRAVEL_DIST: f32 = 320.0;
const DETECT_RANGE: f32 = 260.0;
const PATROL_INTERVAL: f64 = 5.0;
const PATROL_RADIUS: f32 = 300.0;

const POLITICAL_SHIELD_DR: f32 = 0.45; // 45 % reduction

const CARGO_ITEMS: &[&str] = &[
    "smuggled_goods",
    "contraband_rice",
    "black_market_parts",
    "offshore_manifest",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RamState {
    Ready,
    Windup,
    Charging,
}

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct SmuglerVessel {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    attack_damage: i32,

    #[export]
    attack_cooldown: f64,

    #[export]
    #[var(get = get_cargo_count, set = set_cargo_count)]
    cargo_count: i32,

    mob_state: MobState,

    can_slash: bool,
    slash_timer: f64,

    ram_state: RamState,
    ram_windup_timer: f64,
    ram_cooldown_timer: f64,
    ram_heading: Vector2,
    ram_origin: Vector2,

    patrol_target: Vector2,
    patrol_timer: f64,

    shield_active: bool,
}

#[godot_api]
impl ICharacterBody2D for SmuglerVessel {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: MAX_HP,
            attack_damage: 9,
            attack_cooldown: 2.0,
            cargo_count: 3,
            mob_state: MobState::Idle,
            can_slash: true,
            slash_timer: 0.0,
            ram_state: RamState::Ready,
            ram_windup_timer: 0.0,
            ram_cooldown_timer: 0.0,
            ram_heading: Vector2::ZERO,
            ram_origin: Vector2::ZERO,
            patrol_target: Vector2::ZERO,
            patrol_timer: 0.0,
            shield_active: false,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("enemy");
        self.base_mut().add_to_group("sea_mob");
        let pos = self.base_mut().get_global_position();
        self.patrol_target = pos;
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.shield_active = self.check_political_shield();
        self.tick_attack_cooldown(delta);
        self.tick_ram_cooldown(delta);

        // If mid-charge, keep going regardless of player position.
        if self.ram_state == RamState::Charging {
            self.process_ram_charge(delta);
            return;
        }

        let my_pos = self.base_mut().get_global_position();
        let Some(player_node) = self
            .base_mut()
            .get_tree()
            .get_nodes_in_group("player")
            .get(0)
        else {
            self.process_patrol(delta);
            return;
        };
        let Ok(player_gd) = player_node.try_cast::<CharacterBody2D>() else {
            self.process_patrol(delta);
            return;
        };
        let player_pos = player_gd.get_global_position();
        let distance = my_pos.distance_to(player_pos);

        if distance > DETECT_RANGE {
            self.mob_state = MobState::Idle;
            self.process_patrol(delta);
            return;
        }

        self.mob_state = MobState::Aggro;

        if self.ram_state == RamState::Windup {
            self.ram_windup_timer += delta;
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
            if self.ram_windup_timer >= RAM_WINDUP {
                self.begin_ram_charge(my_pos, player_pos);
            }
            return;
        }

        if distance <= RAM_RANGE
            && self.ram_state == RamState::Ready
            && self.ram_cooldown_timer <= 0.0
        {
            self.ram_state = RamState::Windup;
            self.ram_windup_timer = 0.0;
            self.ram_heading = (player_pos - my_pos).normalized();
            self.base_mut().emit_signal("ram_incoming", &[]);
            godot_print!("Smuggler Vessel: RAM INCOMING — lining up!");
            return;
        }

        self.chase(player_pos, CHASE_SPEED);

        if distance <= 48.0 && self.can_slash {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                let dmg = self.attack_damage;
                player.bind_mut().take_damage(dmg);
                godot_print!("Smuggler Vessel scrapes player for {}!", dmg);
            }
            self.can_slash = false;
            self.slash_timer = 0.0;
        }
    }
}

impl Entity for SmuglerVessel {
    fn take_damage(&mut self, amount: i32) {
        let effective = if self.shield_active {
            let reduced = (amount as f32 * (1.0 - POLITICAL_SHIELD_DR)) as i32;
            godot_print!(
                "Smuggler Vessel: political shield active — {} → {} damage.",
                amount,
                reduced
            );
            reduced.max(1)
        } else {
            amount
        };

        self.health = (self.health - effective).max(0);
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

impl HostileBehavior for SmuglerVessel {
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
impl SmuglerVessel {
    #[signal]
    fn ram_incoming();

    #[signal]
    fn ram_hit(damage: i32);

    #[signal]
    fn cargo_dropped(item_id: GString, position: Vector2);

    #[signal]
    fn political_shield_triggered();

    fn tick_attack_cooldown(&mut self, delta: f64) {
        if !self.can_slash {
            self.slash_timer += delta;
            if self.slash_timer >= self.attack_cooldown {
                self.can_slash = true;
                self.slash_timer = 0.0;
            }
        }
    }

    fn tick_ram_cooldown(&mut self, delta: f64) {
        if self.ram_cooldown_timer > 0.0 {
            self.ram_cooldown_timer -= delta;
        }
    }

    fn begin_ram_charge(&mut self, origin: Vector2, target: Vector2) {
        self.ram_state = RamState::Charging;
        self.ram_origin = origin;
        self.ram_heading = (target - origin).normalized();
        godot_print!("Smuggler Vessel: CHARGING!");
    }

    fn process_ram_charge(&mut self, _delta: f64) {
        let my_pos = self.base_mut().get_global_position();
        let travelled = my_pos.distance_to(self.ram_origin);

        if travelled >= RAM_TRAVEL_DIST {
            // Ram finished — reset state.
            self.ram_state = RamState::Ready;
            self.ram_cooldown_timer = RAM_COOLDOWN;
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
            godot_print!("Smuggler Vessel: ram complete.");
            return;
        }

        let heading = self.ram_heading;
        self.sprite.set_flip_h(heading.x < 0.0);
        self.base_mut().set_velocity(heading * RAM_CHARGE_SPEED);
        self.base_mut().move_and_slide();

        let players = self.base_mut().get_tree().get_nodes_in_group("player");
        for node in players.iter_shared() {
            if let Ok(player_gd) = node.try_cast::<CharacterBody2D>() {
                if my_pos.distance_to(player_gd.get_global_position()) <= 52.0 {
                    if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                        player.bind_mut().take_damage(RAM_DAMAGE);
                        self.base_mut()
                            .emit_signal("ram_hit", &[Variant::from(RAM_DAMAGE)]);
                        godot_print!("Smuggler Vessel RAMS player for {}!", RAM_DAMAGE);
                    }
                    self.ram_state = RamState::Ready;
                    self.ram_cooldown_timer = RAM_COOLDOWN;
                    self.base_mut().set_velocity(Vector2::ZERO);
                    self.base_mut().move_and_slide();
                    return;
                }
            }
        }
    }

    fn process_patrol(&mut self, delta: f64) {
        self.patrol_timer += delta;
        if self.patrol_timer >= PATROL_INTERVAL {
            self.patrol_timer = 0.0;
            self.pick_patrol_target();
        }

        let target = self.patrol_target;
        let pos = self.base_mut().get_global_position();
        if pos.distance_to(target) < 10.0 {
            self.base_mut().set_velocity(Vector2::ZERO);
        } else {
            let dir = (target - pos).normalized();
            self.sprite.set_flip_h(dir.x < 0.0);
            self.base_mut().set_velocity(dir * PATROL_SPEED);
        }
        self.base_mut().move_and_slide();
    }

    fn pick_patrol_target(&mut self) {
        let pos = self.base_mut().get_global_position();
        let offset = Vector2::new(
            (pseudo_rand() - 0.5) * PATROL_RADIUS * 2.0,
            (pseudo_rand() - 0.5) * PATROL_RADIUS * 2.0,
        );
        self.patrol_target = pos + offset;
    }

    fn check_political_shield(&mut self) -> bool {
        let bosses = self.base_mut().get_tree().get_nodes_in_group("boss");
        let active = !bosses.is_empty();
        if active {
            self.base_mut()
                .emit_signal("political_shield_triggered", &[]);
        }
        active
    }

    fn on_death(&mut self) {
        let pos = self.base_mut().get_global_position();
        let count = self.cargo_count.clamp(0, CARGO_ITEMS.len() as i32) as usize;
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        for item_id in &CARGO_ITEMS[..count] {
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("item_dropped")),
                    Variant::from(GString::from(*item_id)),
                    Variant::from(pos),
                ],
            );
        }
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, MAX_HP);
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }

    #[func]
    pub fn set_cargo_count(&mut self, count: i32) {
        self.cargo_count = count.clamp(0, CARGO_ITEMS.len() as i32);
    }

    #[func]
    pub fn get_cargo_count(&self) -> i32 {
        self.cargo_count
    }

    #[func]
    pub fn is_politically_shielded(&self) -> bool {
        self.shield_active
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
