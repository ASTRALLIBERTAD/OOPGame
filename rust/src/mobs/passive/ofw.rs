use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, MobState, PassiveBehavior};

const MAX_HP: i32 = 30;
const TRAVEL_SPEED: f32 = 65.0;
const FLEE_SPEED: f32 = 120.0;
const FLEE_DURATION: f64 = 3.0;
const FEAR_RADIUS: f32 = 180.0;
const WANDER_RADIUS: f32 = 400.0;
const WANDER_INTERVAL: f64 = 12.0;
const VISIT_DURATION: f64 = 180.0;
const VISIT_WARNING_AT: f64 = 30.0;
const BOX_HEAL_AMOUNT: i32 = 8;
const BOX_HUNGER_RESTORE: i32 = 10;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Ofw {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    #[var(get = get_has_traded)]
    has_traded: bool,

    #[export]
    #[var(get = get_visit_time_remaining)]
    visit_time_remaining: f64,

    #[export]
    stock_id: GString,

    mob_state: MobState,

    wander_target: Vector2,
    wander_timer: f64,

    flee_target: Option<Vector2>,
    flee_timer: f64,

    warning_fired: bool,
    box_opened: bool,
}

#[godot_api]
impl ICharacterBody2D for Ofw {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            health: MAX_HP,
            has_traded: false,
            visit_time_remaining: VISIT_DURATION,
            stock_id: GString::from("ofw_box_default"),
            mob_state: MobState::Idle,
            wander_target: Vector2::ZERO,
            wander_timer: 0.0,
            flee_target: None,
            flee_timer: 0.0,
            warning_fired: false,
            box_opened: false,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("neutral");
        self.base_mut().add_to_group("civilian");
        self.base_mut().add_to_group("trader");
        let pos = self.base_mut().get_global_position();
        self.wander_target = pos;
        godot_print!("OFW arrived. Visit window: {}s.", VISIT_DURATION);
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.tick_visit_timer(delta);

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
        if self.wander_timer >= WANDER_INTERVAL {
            self.wander_timer = 0.0;
            self.wander();
        }
        let target = self.wander_target;
        self.move_toward(target, TRAVEL_SPEED);
    }
}

impl Entity for Ofw {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            let pos = self.base_mut().get_global_position();
            self.base_mut()
                .emit_signal("balikbayan_box_dropped", &[Variant::from(pos)]);
            godot_print!("The OFW was killed. Their box is on the ground.");
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

impl PassiveBehavior for Ofw {
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
        self.flee_target = Some(pos + away * 300.0);
        self.flee_timer = 0.0;
        self.mob_state = MobState::Fleeing;
        godot_print!("OFW is fleeing — they didn't come home for this.");
    }
}

#[godot_api]
impl Ofw {
    #[signal]
    fn trade_opened(stock_id: GString);

    #[signal]
    fn visit_almost_over(seconds_left: f64);

    #[signal]
    fn visit_ended();

    #[signal]
    fn balikbayan_box_dropped(position: Vector2);

    #[signal]
    fn box_contents_used(heal: i32, hunger: i32);

    #[func]
    pub fn on_interact(&mut self) {
        if self.has_traded {
            godot_print!("OFW: 'I already gave you everything I brought.'");
            return;
        }

        self.has_traded = true;
        let stock = self.stock_id.clone();
        self.base_mut()
            .emit_signal("trade_opened", &[Variant::from(stock)]);
        godot_print!(
            "OFW opens their box. Visit time remaining: {:.0}s.",
            self.visit_time_remaining
        );
    }

    #[func]
    pub fn open_box(&mut self) {
        if self.box_opened {
            godot_print!("OFW: 'The box is already open.'");
            return;
        }

        self.box_opened = true;

        let tree = self.base_mut().get_tree();
        let players = tree.get_nodes_in_group("player");

        for node in players.iter_shared() {
            if let Ok(mut player_gd) = node.try_cast::<crate::rustplayer::Rustplayer>() {
                player_gd.bind_mut().heal(BOX_HEAL_AMOUNT);
                player_gd.bind_mut().feed(BOX_HUNGER_RESTORE);
                self.base_mut().emit_signal(
                    "box_contents_used",
                    &[
                        Variant::from(BOX_HEAL_AMOUNT),
                        Variant::from(BOX_HUNGER_RESTORE),
                    ],
                );
                godot_print!(
                    "OFW box used: +{} HP, +{} hunger.",
                    BOX_HEAL_AMOUNT,
                    BOX_HUNGER_RESTORE
                );
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
    pub fn get_has_traded(&self) -> bool {
        self.has_traded
    }

    #[func]
    pub fn get_visit_time_remaining(&self) -> f64 {
        self.visit_time_remaining
    }

    fn tick_visit_timer(&mut self, delta: f64) {
        self.visit_time_remaining -= delta;

        if !self.warning_fired && self.visit_time_remaining <= VISIT_WARNING_AT {
            self.warning_fired = true;
            let visit_time_remaining = self.visit_time_remaining;
            self.base_mut()
                .emit_signal("visit_almost_over", &[Variant::from(visit_time_remaining)]);
            godot_print!("OFW: 'I have to leave soon. Anything else?'");
        }

        if self.visit_time_remaining <= 0.0 {
            self.base_mut().emit_signal("visit_ended", &[]);
            godot_print!("OFW: 'Goodbye. I'll send money when I get there.'");
            self.base_mut().queue_free();
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
