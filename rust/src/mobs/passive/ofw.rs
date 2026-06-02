use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;

use crate::entity::{Entity, MobState, PassiveBehavior};

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

    #[export]
    travel_speed: f32,

    #[export]
    flee_speed: f32,

    #[export]
    flee_duration: f64,

    #[export]
    fear_radius: f32,

    #[export]
    wander_radius: f32,

    #[export]
    wander_interval: f64,

    #[export]
    visit_duration: f64,

    #[export]
    visit_warning_at: f64,

    #[export]
    box_heal_amount: i32,

    #[export]
    box_hunger_restore: i32,

    mob_state: MobState,
    home_position: Vector2,
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
            health: 30,
            has_traded: false,
            visit_time_remaining: 180.0,
            stock_id: GString::from("ofw_box_default"),
            travel_speed: 65.0,
            flee_speed: 120.0,
            flee_duration: 3.0,
            fear_radius: 180.0,
            wander_radius: 400.0,
            wander_interval: 12.0,
            visit_duration: 180.0,
            visit_warning_at: 30.0,
            box_heal_amount: 8,
            box_hunger_restore: 10,
            mob_state: MobState::Idle,
            home_position: Vector2::ZERO,
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
        self.home_position = pos;
        godot_print!("OFW arrived. Visit window: {}s.", self.visit_duration);
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.tick_visit_timer(delta);

        if self.mob_state == MobState::Fleeing {
            self.flee_timer += delta;
            if self.flee_timer >= self.flee_duration {
                self.flee_timer = 0.0;
                self.flee_target = None;
                self.mob_state = MobState::Idle;
                self.wander();
            } else if let Some(fp) = self.flee_target {
                self.move_toward(fp, self.flee_speed);
                return;
            }
        }

        if let Some(threat) = self.nearest_threat_position() {
            if let Some(priest_pos) = self.nearest_priest_position() {
                self.flee_to(priest_pos);
            } else {
                self.flee(threat);
            }
            return;
        }

        self.wander_timer += delta;
        if self.wander_timer >= self.wander_interval {
            self.wander_timer = 0.0;
            self.wander();
        }
        let target = self.wander_target;
        self.move_toward(target, self.travel_speed);
    }
}

impl Entity for Ofw {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            let pos = self.base_mut().get_global_position();
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("balikbayan_box_dropped")),
                    Variant::from(pos),
                ],
            );
            godot_print!("The OFW was killed. Their box is on the ground.");
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).clamp(0, 30);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl PassiveBehavior for Ofw {
    fn wander(&mut self) {
        let angle = (godot::global::randf() * std::f64::consts::TAU) as f32;
        let dist = godot::global::randf() as f32 * self.wander_radius;
        let offset = Vector2::new(angle.cos() * dist, angle.sin() * dist);
        self.wander_target = self.home_position + offset;
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
    fn box_contents_used(heal: i32, hunger: i32);

    fn flee_to(&mut self, destination: Vector2) {
        self.flee_target = Some(destination);
        self.flee_timer = 0.0;
        self.mob_state = MobState::Fleeing;
        godot_print!("OFW runs to the Priest!");
    }

    fn nearest_priest_position(&mut self) -> Option<Vector2> {
        let my_pos = self.base_mut().get_global_position();
        let healers = self.base_mut().get_tree().get_nodes_in_group("healer");
        let mut nearest: Option<(f32, Vector2)> = None;
        for node in healers.iter_shared() {
            if let Ok(body) = node.try_cast::<CharacterBody2D>() {
                let pos = body.get_global_position();
                let dist = my_pos.distance_to(pos);
                if nearest.is_none_or(|(d, _)| dist < d) {
                    nearest = Some((dist, pos));
                }
            }
        }
        nearest.map(|(_, pos)| pos)
    }

    fn nearest_threat_position(&mut self) -> Option<Vector2> {
        let my_pos = self.base_mut().get_global_position();
        let enemies = self.base_mut().get_tree().get_nodes_in_group("enemy");
        let mut nearest: Option<(f32, Vector2)> = None;
        for enemy in enemies.iter_shared() {
            if let Ok(body) = enemy.try_cast::<CharacterBody2D>() {
                let epos = body.get_global_position();
                let dist = my_pos.distance_to(epos);
                if dist <= self.fear_radius && nearest.is_none_or(|(d, _)| dist < d) {
                    nearest = Some((dist, epos));
                }
            }
        }
        nearest.map(|(_, pos)| pos)
    }

    fn move_toward(&mut self, target: Vector2, speed: f32) {
        let pos = self.base_mut().get_global_position();
        if pos.distance_to(target) < 6.0 {
            if speed == self.travel_speed {
                self.wander_timer = self.wander_interval;
            }
            self.base_mut().set_velocity(Vector2::ZERO);
        } else {
            let dir = (target - pos).normalized();
            self.sprite.set_flip_h(dir.x < 0.0);
            self.base_mut().set_velocity(dir * speed);
        }
        self.base_mut().move_and_slide();
    }

    fn tick_visit_timer(&mut self, delta: f64) {
        self.visit_time_remaining -= delta;

        if !self.warning_fired && self.visit_time_remaining <= self.visit_warning_at {
            self.warning_fired = true;
            let t = self.visit_time_remaining;
            self.base_mut()
                .emit_signal("visit_almost_over", &[Variant::from(t)]);
            godot_print!("OFW: 'I have to leave soon. Anything else?'");
        }

        if self.visit_time_remaining <= 0.0 {
            self.base_mut().emit_signal("visit_ended", &[]);
            godot_print!("OFW: 'Goodbye. I'll send money when I get there.'");
            self.base_mut().queue_free();
        }
    }

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
        let heal = self.box_heal_amount;
        let hunger = self.box_hunger_restore;
        let tree = self.base_mut().get_tree();
        let players = tree.get_nodes_in_group("player");
        for node in players.iter_shared() {
            if let Ok(mut player_gd) = node.try_cast::<crate::rustplayer::Rustplayer>() {
                player_gd.bind_mut().heal(heal);
                player_gd.bind_mut().feed(hunger);
                self.base_mut().emit_signal(
                    "box_contents_used",
                    &[Variant::from(heal), Variant::from(hunger)],
                );
                godot_print!("OFW box used: +{} HP, +{} hunger.", heal, hunger);
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
        self.health = health.clamp(0, 30);
    }

    #[func]
    pub fn get_has_traded(&self) -> bool {
        self.has_traded
    }

    #[func]
    pub fn get_visit_time_remaining(&self) -> f64 {
        self.visit_time_remaining
    }
}
