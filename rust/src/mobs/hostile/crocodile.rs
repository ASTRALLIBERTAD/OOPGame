use godot::classes::{AnimatedSprite2D, Area2D, CharacterBody2D, ICharacterBody2D};
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::entity::{Entity, HostileBehavior, MobState};
use crate::rustplayer::Rustplayer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuwayaPhase {
    Phase1,
    Phase2,
    Phase3,
}

const MAX_HP: i32 = 300;
const PHASE2_THRESHOLD: i32 = 200;
const PHASE3_THRESHOLD: i32 = 100;
const PHASE3_HP_CAP: i32 = 150;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Crocodile {
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
    #[var(get = get_corruption_tiles, set = set_corruption_tiles)]
    corruption_tiles: i32,

    mob_state: MobState,
    phase: BuwayaPhase,

    can_slash: bool,
    slash_timer: f64,

    bribe_resolved: bool,
    bribe_cooldown: f64,
    bribe_timer: f64,

    reinforcements_spawned: bool,

    regen_timer: f64,

    #[export]
    troll_scene: OnEditor<Gd<PackedScene>>,

    #[export]
    enforcer_scene: OnEditor<Gd<PackedScene>>,
}

#[godot_api]
impl ICharacterBody2D for Crocodile {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            attack_area: OnEditor::default(),
            health: MAX_HP,
            speed: 55.0,
            aggro_range: 320.0,
            attack_damage: 15,
            attack_cooldown: 1.4,
            corruption_tiles: 0,
            mob_state: MobState::Idle,
            phase: BuwayaPhase::Phase1,
            can_slash: true,
            slash_timer: 0.0,
            bribe_resolved: false,
            bribe_cooldown: 10.0,
            bribe_timer: 0.0,
            reinforcements_spawned: false,
            regen_timer: 0.0,
            troll_scene: OnEditor::default(),
            enforcer_scene: OnEditor::default(),
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("enemy");
        self.base_mut().add_to_group("boss");
    }

    fn process(&mut self, delta: f64) {
        if !self.is_alive() {
            return;
        }

        self.tick_phase_transitions();
        self.tick_attack_cooldown(delta);
        self.tick_phase3_regen(delta);

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

        if self.phase == BuwayaPhase::Phase1 && !self.bribe_resolved {
            self.tick_bribe(delta);
        }

        if self.phase == BuwayaPhase::Phase2 && !self.reinforcements_spawned {
            self.call_reinforcements();
        }

        self.chase(player_pos, self.speed);

        if distance <= 55.0 && self.can_slash {
            if let Ok(mut player) = player_gd.try_cast::<Rustplayer>() {
                let dmg = self.attack_damage;
                player.bind_mut().take_damage(dmg);
                godot_print!("Buwaya strikes for {} damage!", dmg);
            }
            self.can_slash = false;
            self.slash_timer = 0.0;
        }
    }
}

impl Entity for Crocodile {
    fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        godot_print!("Buhaya Health: {}", self.health);
        if !self.is_alive() {
            self.mob_state = MobState::Dead;
            self.on_death();
            self.base_mut().queue_free();
        }
    }

    fn heal(&mut self, amount: i32) {
        let cap = if self.phase == BuwayaPhase::Phase3 {
            PHASE3_HP_CAP
        } else {
            MAX_HP
        };
        self.health = (self.health + amount).min(cap);
    }

    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

impl HostileBehavior for Crocodile {
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
impl Crocodile {
    #[signal]
    fn bribe_offered(piso: i32);

    #[signal]
    fn boss_defeated();

    #[signal]
    fn drop_item(item_id: GString, position: Vector2);

    fn tick_phase_transitions(&mut self) {
        let new_phase = if self.health > PHASE2_THRESHOLD {
            BuwayaPhase::Phase1
        } else if self.health > PHASE3_THRESHOLD {
            BuwayaPhase::Phase2
        } else {
            BuwayaPhase::Phase3
        };

        if new_phase != self.phase {
            self.phase = new_phase;
            match self.phase {
                BuwayaPhase::Phase2 => {
                    godot_print!("Buwaya: calling his enforcers...");
                }
                BuwayaPhase::Phase3 => {
                    godot_print!("Buwaya reveals his true form!");
                    self.speed *= 1.3;
                    self.attack_damage += 5;
                }
                _ => {}
            }
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

    fn tick_phase3_regen(&mut self, delta: f64) {
        if self.phase != BuwayaPhase::Phase3 || self.corruption_tiles <= 0 {
            self.regen_timer = 0.0;
            return;
        }
        self.regen_timer += delta;
        if self.regen_timer >= 2.0 {
            self.regen_timer = 0.0;
            let regen = (self.corruption_tiles * 2).min(10);
            self.heal(regen);
            godot_print!(
                "Buwaya regens {} HP from {} corruption tiles",
                regen,
                self.corruption_tiles
            );
        }
    }

    fn tick_bribe(&mut self, delta: f64) {
        self.bribe_timer += delta;
        if self.bribe_timer >= self.bribe_cooldown {
            self.bribe_timer = 0.0;
            self.base_mut()
                .emit_signal("bribe_offered", &[Variant::from(500_i32)]);
            godot_print!("Buwaya: 'Let us not fight. I have an offer for you.'");
        }
    }

    fn call_reinforcements(&mut self) {
        self.reinforcements_spawned = true;
        godot_print!("Buwaya calls his troll army and enforcers!");

        let my_pos = self.base_mut().get_global_position();
        let mut parent = self.base_mut().get_parent().unwrap();

        let troll_offsets = [
            Vector2::new(-120.0, 0.0),
            Vector2::new(120.0, 0.0),
            Vector2::new(0.0, -120.0),
        ];

        for offset in troll_offsets {
            let mut instance = self.troll_scene.instantiate().unwrap().cast::<Node2D>();
            instance.set_global_position(my_pos + offset);
            parent.add_child(&instance);
        }

        let enforcer_offsets = [Vector2::new(-180.0, 80.0), Vector2::new(180.0, 80.0)];

        for offset in enforcer_offsets {
            let mut instance = self.enforcer_scene.instantiate().unwrap().cast::<Node2D>();
            instance.set_global_position(my_pos + offset);
            parent.add_child(&instance);
        }
    }

    fn on_death(&mut self) {
        godot_print!("The Buwaya falls... but the system remains.");
        let pos = self.base_mut().get_global_position();
        self.base_mut().emit_signal(
            "drop_item",
            &[
                Variant::from(GString::from("barong_of_authority")),
                Variant::from(pos),
            ],
        );
        self.base_mut().emit_signal(
            "drop_item",
            &[
                Variant::from(GString::from("seal_of_reform")),
                Variant::from(pos),
            ],
        );
        self.base_mut().emit_signal(
            "drop_item",
            &[
                Variant::from(GString::from("black_ledger")),
                Variant::from(pos),
            ],
        );
        self.base_mut().emit_signal("boss_defeated", &[]);
    }

    fn get_player(&mut self) -> Option<Gd<Rustplayer>> {
        let node = self
            .base_mut()
            .get_tree()
            .get_nodes_in_group("player")
            .get(0)?;
        node.try_cast::<CharacterBody2D>()
            .ok()
            .and_then(|b| b.try_cast::<Rustplayer>().ok())
    }

    #[func]
    pub fn on_bribe_accepted(&mut self) {
        self.bribe_resolved = true;
        if let Some(mut player) = self.get_player() {
            player.bind_mut().apply_indebted();
        }
        godot_print!("Buwaya: 'Smart choice. This is how the world works.'");
    }

    #[func]
    pub fn on_bribe_rejected(&mut self) {
        self.bribe_resolved = true;
        self.attack_damage += 3;
        godot_print!("Buwaya: 'You will regret this.'");
    }

    #[func]
    pub fn set_corruption_tiles(&mut self, count: i32) {
        self.corruption_tiles = count.max(0);
    }

    #[func]
    pub fn get_corruption_tiles(&self) -> i32 {
        self.corruption_tiles
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
