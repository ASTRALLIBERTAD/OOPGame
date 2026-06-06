use godot::classes::{AnimatedSprite2D, Area2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, HostileBehavior, MobState};
use crate::rustplayer::Rustplayer;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Troll {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    disinfo_aura: OnEditor<Gd<Area2D>>,

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
    confused_duration: f64,

    #[export]
    aura_tick_rate: f64,

    mob_state: MobState,
    can_slash: bool,
    slash_timer: f64,
    aura_timer: f64,
    playing_oneshot: bool,
    flash_timer: f64,
}

#[godot_api]
impl ICharacterBody2D for Troll {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            disinfo_aura: OnEditor::default(),
            health: 25,
            speed: 90.0,
            aggro_range: 200.0,
            attack_damage: 3,
            attack_cooldown: 1.5,
            confused_duration: 4.0,
            aura_tick_rate: 2.5,
            mob_state: MobState::Idle,
            can_slash: true,
            slash_timer: 0.0,
            aura_timer: 0.0,
            playing_oneshot: false,
            flash_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("enemy");
        self.base_mut().add_to_group("troll_pack");
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

        if !self.can_slash {
            self.slash_timer += delta;
            if self.slash_timer >= self.attack_cooldown {
                self.can_slash = true;
                self.slash_timer = 0.0;
            }
        }

        let Some((target_gd, distance)) = self.nearest_target() else {
            return;
        };
        let target_pos = target_gd.get_global_position();

        if distance > self.aggro_range {
            self.mob_state = MobState::Idle;
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
            if !self.playing_oneshot && self.sprite.get_animation().to_string() != "default" {
                self.sprite.play_ex().name("default").done();
            }
            return;
        }

        self.aggro(target_pos);

        if self.playing_oneshot {
            self.base_mut().set_velocity(Vector2::ZERO);
            self.base_mut().move_and_slide();
            return;
        }

        self.chase(target_pos, self.speed);

        self.aura_timer += delta;
        if self.aura_timer >= self.aura_tick_rate {
            self.aura_timer = 0.0;
            self.apply_confused_to_nearby_players();
        }

        if distance <= 35.0 && self.can_slash {
            let dmg = self.attack_damage;
            if let Ok(mut player) = target_gd.clone().try_cast::<Rustplayer>() {
                player.bind_mut().take_damage(dmg);
            } else {
                self.deal_damage_to_civilian(target_gd, dmg);
            }
            self.playing_oneshot = true;
            self.sprite.play_ex().name("attack").done();
            self.can_slash = false;
            self.slash_timer = 0.0;
        }
    }
}

impl Entity for Troll {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        self.base_mut().set_modulate(Color::from_rgb(1.0, 0.3, 0.3));
        self.flash_timer = 0.2;
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.playing_oneshot = true;
            self.sprite.play_ex().name("death").done();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(25);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl HostileBehavior for Troll {
    fn aggro(&mut self, _target: Vector2) {
        self.mob_state = MobState::Aggro;
    }

    fn chase(&mut self, target: Vector2, speed: f32) {
        let pos = self.base_mut().get_global_position();
        let dir = (target - pos).normalized();
        self.sprite.set_flip_h(dir.x < 0.0);
        self.base_mut().set_velocity(dir * speed);
        self.base_mut().move_and_slide();
        if self.sprite.get_animation().to_string() != "walking_running" {
            self.sprite.play_ex().name("walking_running").done();
        }
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
impl Troll {
    fn nearest_target(&mut self) -> Option<(Gd<CharacterBody2D>, f32)> {
        let my_pos = self.base_mut().get_global_position();
        let mut nearest: Option<(Gd<CharacterBody2D>, f32)> = None;
        for group in ["player", "civilian", "neutral"] {
            for node in self
                .base_mut()
                .get_tree()
                .get_nodes_in_group(group)
                .iter_shared()
            {
                if let Ok(body) = node.try_cast::<CharacterBody2D>() {
                    let dist = my_pos.distance_to(body.get_global_position());
                    if nearest.as_ref().map_or(true, |(_, d)| dist < *d) {
                        nearest = Some((body, dist));
                    }
                }
            }
        }
        nearest
    }

    fn deal_damage_to_civilian(&mut self, body: Gd<CharacterBody2D>, damage: i32) {
        if let Ok(mut farmer) = body
            .clone()
            .try_cast::<crate::mobs::passive::farmer::Farmer>()
        {
            farmer.bind_mut().take_damage(damage);
        } else if let Ok(mut priest) = body
            .clone()
            .try_cast::<crate::mobs::passive::priest::Priest>()
        {
            priest.bind_mut().take_damage(damage);
        } else if let Ok(mut ofw) = body.clone().try_cast::<crate::mobs::passive::ofw::Ofw>() {
            ofw.bind_mut().take_damage(damage);
        } else if let Ok(mut trader) =
            body.clone()
                .try_cast::<crate::mobs::neutral::roaming_trader::RoamingTrader>()
        {
            trader.bind_mut().take_damage(damage);
        } else if let Ok(mut student) = body
            .clone()
            .try_cast::<crate::mobs::neutral::student::Student>()
        {
            student.bind_mut().take_damage(damage);
        } else if let Ok(mut journalist) =
            body.try_cast::<crate::mobs::neutral::journalist::Journalist>()
        {
            journalist.bind_mut().take_damage(damage);
        }
    }

    fn apply_confused_to_nearby_players(&mut self) {
        let duration = self.confused_duration;
        for body in self.disinfo_aura.get_overlapping_bodies().iter_shared() {
            if body.is_in_group("player") {
                if let Ok(mut player) = body.try_cast::<Rustplayer>() {
                    if !player.bind().is_confused() {
                        player.bind_mut().apply_confused(duration);
                        godot_print!("Troll applies Confused debuff for {}s!", duration);
                    }
                }
                break;
            }
        }
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, 25);
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
