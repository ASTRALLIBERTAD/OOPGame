use godot::classes::{
    AnimatedSprite2D, Area2D, BoxContainer, Camera2D, CanvasLayer, CharacterBody2D, Control,
    ICharacterBody2D, Input, Label,
};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;
use std::str::FromStr;

use crate::armor_system::ArmorSystem;
use crate::entity::Entity;
use crate::heart::Heart;
use crate::inv_slot::InvSlot;
use crate::inventory::Inventory;
use crate::item_collectibles::Collectibles;
use crate::node_manager::NodeManager;

const MAX_HEALTH: i32 = 20;
const ATTACK_DAMAGE: i32 = 10;
const ATTACK_COOLDOWN: f64 = 0.6;
const INDEBTED_ATTACK_REDUCTION: i32 = 3;
const INDEBTED_DURATION: f64 = 60.0;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
pub struct Rustplayer {
    #[base]
    base: Base<CharacterBody2D>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,

    #[export]
    coords: OnEditor<Gd<Label>>,

    #[export]
    inv: OnEditor<Gd<Inventory>>,

    #[export]
    armor_system: OnEditor<Gd<ArmorSystem>>,

    #[export]
    item_slot: OnEditor<Gd<Control>>,

    is_open: bool,

    #[export]
    #[var(get = get_heart_ui)]
    heart_ui: OnEditor<Gd<Heart>>,

    #[export]
    #[var(get = get_health, set = set_health)]
    health: i32,

    #[export]
    camera: OnEditor<Gd<Camera2D>>,

    target_position: Vector2,

    #[export]
    pub id: i32,

    last_chunk_pos: Vector2i,
    last_update_time: f64,

    #[export]
    item_right: OnEditor<Gd<InvSlot>>,

    #[export]
    hotbar: OnEditor<Gd<BoxContainer>>,

    #[export]
    touch_control: OnEditor<Gd<CanvasLayer>>,

    can_slash: bool,
    slash_timer: f64,

    #[export]
    attack_area: OnEditor<Gd<Area2D>>,

    #[export]
    #[var(get = get_hunger, set = set_hunger)]
    hunger: i32,

    hunger_drain_timer: f64,
    regen_timer: f64,
    starvation_timer: f64,

    indebted: bool,
    indebted_timer: f64,

    confused: bool,
    confused_timer: f64,

    arrested: bool,
    arrested_timer: f64,

    piso: i32,

    in_sanctuary: bool,
    blessed: bool,
    blessed_timer: f64,
    speed_bonus: f32,
    speed_bonus_timer: f64,

    playing_oneshot: bool,
    flash_timer: f64,
}

#[godot_api]
impl ICharacterBody2D for Rustplayer {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            coords: OnEditor::default(),
            inv: OnEditor::default(),
            armor_system: OnEditor::default(),
            item_slot: OnEditor::default(),
            is_open: false,
            heart_ui: OnEditor::default(),
            health: MAX_HEALTH,
            camera: OnEditor::default(),
            target_position: Vector2::default(),
            id: i32::default(),
            last_chunk_pos: Vector2i::new(i32::MAX, i32::MAX),
            last_update_time: 0.0,
            item_right: OnEditor::default(),
            hotbar: OnEditor::default(),
            touch_control: OnEditor::default(),
            can_slash: true,
            slash_timer: 0.0,
            attack_area: OnEditor::default(),
            hunger: 20,
            hunger_drain_timer: 0.0,
            regen_timer: 0.0,
            starvation_timer: 0.0,
            indebted: false,
            indebted_timer: 0.0,

            confused: false,
            confused_timer: 0.0,

            arrested: false,
            arrested_timer: 0.0,

            piso: 200,

            in_sanctuary: false,
            blessed: false,
            blessed_timer: 0.0,
            speed_bonus: 0.0,
            speed_bonus_timer: 0.0,

            playing_oneshot: false,
            flash_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group("player");

        let pid = self.base_mut().get_multiplayer_authority();
        self.id = pid;

        godot_print!("Player ID is : {}", self.id);
        let is_authority = self.base_mut().is_multiplayer_authority();

        let mut label_piso = self
            .base()
            .get_node_as::<Label>("Control/CanvasLayer/VBoxContainer/piso");
        let piso = self.piso.to_string();
        label_piso.set_text(&piso);

        if !is_authority {
            self.camera.make_current();
            self.heart_ui.bind_mut().set_heart_display(self.health);
        }

        self.attack_area.set_monitoring(true);
        self.attack_area.set_monitorable(false);

        let callable = self.base().callable("on_animation_finished");
        self.sprite.connect("animation_finished", &callable);
    }

    fn process(&mut self, delta: f64) {
        if self.flash_timer > 0.0 {
            self.flash_timer -= delta;
            if self.flash_timer <= 0.0 {
                self.flash_timer = 0.0;
                self.base_mut().set_modulate(Color::WHITE);
            }
        }

        if self.base_mut().is_multiplayer_authority() {
            let slots = self.inv.bind().get_slots();
            let armor_speed_mod = self.armor_system.bind().total_speed_modifier(slots);
            let speed: f32 = (100.0 + self.speed_bonus) * (1.0 + armor_speed_mod);
            let input = Input::singleton();

            let direction = Input::get_vector(
                &input,
                &StringName::from_str("left").unwrap(),
                &StringName::from_str("right").unwrap(),
                &StringName::from_str("up").unwrap(),
                &StringName::from_str("down").unwrap(),
            );

            let velocity = if self.confused {
                direction * speed * -1.0
            } else {
                direction * speed
            };

            if input.is_action_just_pressed(&StringName::from_str("left").unwrap()) {
                self.sprite.set_flip_h(true);
            }
            if input.is_action_just_pressed(&StringName::from_str("right").unwrap()) {
                self.sprite.set_flip_h(false);
            }

            if self.arrested {
                self.base_mut().set_velocity(Vector2::ZERO);
                self.base_mut().move_and_slide();
            } else {
                self.base_mut().set_velocity(velocity);
                self.base_mut().move_and_slide();
            }

            if !self.playing_oneshot {
                let is_moving = self.base_mut().get_velocity().length() > 1.0;
                let current_anim = self.sprite.get_animation().to_string();
                if is_moving {
                    if current_anim != "walking_running" {
                        self.sprite.play_ex().name("walking_running").done();
                    }
                } else {
                    if current_anim != "default" {
                        self.sprite.play_ex().name("default").done();
                    }
                }
            }

            self.update_terrain_if_needed(delta);

            let cord = self.get_player_cord_for_display();
            let y_value = if cord.y == 0.0 { cord.y * 1.0 } else { -cord.y };

            let k = format!("coordinates :{}, {:?}", cord.x, y_value as i32);
            self.coords.set_text(&k);

            if input.is_action_just_pressed("inventory") {
                if self.is_open {
                    self.close();
                } else {
                    self.open();
                }
            }

            let pos = self.base_mut().get_global_position();
            self.base_mut()
                .rpc("update_position", &[Variant::from(pos)]);

            if input.is_action_just_pressed("attack") && self.can_slash {
                self.can_slash = false;
                self.slash_timer = 0.0;
                self.attack();
                godot_print!("Player {} performed an attack!", self.id);
            }

            self.tick_attack_cooldown(delta);
        } else {
            let pos = self.target_position;
            let smooth_position = self
                .base_mut()
                .get_global_position()
                .lerp(pos, 10.0 * delta as f32);
            self.base_mut().set_global_position(smooth_position);

            self.last_update_time += delta;
            if self.last_update_time >= 0.5 {
                self.update_terrain_if_needed(delta);
                self.last_update_time = 0.0;
            }
        }

        if self.base_mut().is_multiplayer_authority() {
            self.tick_hunger(delta);
            self.tick_indebted(delta);
            self.tick_confused(delta);
            self.tick_arrested(delta);
            self.tick_blessed(delta);
            self.tick_speed_bonus(delta);
        }
    }
}

impl Entity for Rustplayer {
    fn take_damage(&mut self, amount: i32) {
        if !self.is_alive() {
            return;
        }
        if self.in_sanctuary {
            godot_print!("Sanctuary protects the player!");
            return;
        }
        let slots = self.inv.bind().get_slots();
        let reduction = self.armor_system.bind().total_defense(slots.clone());
        let actual_damage = (amount - reduction).max(0);

        self.armor_system.bind().damage_durability(slots, 1);

        self.health = (self.health - actual_damage).max(0);
        self.heart_ui.bind_mut().set_heart_display(self.health);

        self.base_mut().set_modulate(Color::from_rgb(1.0, 0.3, 0.3));
        self.flash_timer = 0.2;

        self.inv.bind_mut().signals().update().emit();

        if !self.is_alive() {
            godot_print!("player dead");
            // self.playing_oneshot = true;
            // self.sprite.play_ex().name("death").done();
        }
    }

    fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).clamp(0, MAX_HEALTH);
        self.heart_ui.bind_mut().set_heart_display(self.health);
    }
    fn is_alive(&self) -> bool {
        self.health > 0
    }
}

#[godot_api]
impl Rustplayer {
    #[signal]
    fn message(text: String);
    #[signal]
    fn piso_changed(new_total: i32);

    #[func]
    #[rpc(unreliable, any_peer)]
    fn update_position(&mut self, pos: Vector2) {
        self.target_position = pos;
    }

    #[func]
    pub fn tester(&mut self, amount: i32) {
        godot_print!("connected and the amount is : {}", amount);
    }

    fn get_player_cord_for_display(&mut self) -> Vector2 {
        let mut scene = get_autoload_by_name::<NodeManager>("GlobalNodeManager");
        let scene = scene.bind_mut().get_terrain();
        let local_position = self.base_mut().get_global_position();
        let cord = scene.local_to_map(local_position);
        scene.to_local(Vector2::new(cord.x as f32, cord.y as f32))
    }

    fn update_terrain_if_needed(&mut self, _delta: f64) {
        let mut scene = get_autoload_by_name::<NodeManager>("GlobalNodeManager");
        let mut scene = scene.bind_mut().get_terrain();

        let pos = self.base_mut().get_global_position();
        let current_chunk = scene.local_to_map(pos);

        if current_chunk != self.last_chunk_pos {
            let id = self.base_mut().get_multiplayer_authority();
            scene.bind_mut().update_player_position(id, current_chunk);
            self.last_chunk_pos = current_chunk;
        }
    }

    fn open(&mut self) {
        self.is_open = true;
        self.item_slot.set_visible(true);

        self.hotbar.set_visible(false);
        self.touch_control.set_visible(false);
        self.coords.set_visible(false);
        self.heart_ui.set_visible(false);
    }

    fn close(&mut self) {
        self.is_open = false;
        self.item_slot.set_visible(false);

        self.hotbar.set_visible(true);

        self.touch_control.set_visible(true);
        self.coords.set_visible(true);
        self.heart_ui.set_visible(true);
    }

    #[func]
    fn collect_items(&mut self, items: Gd<Collectibles>, index: i32) {
        self.inv.bind_mut().insert(items, index, index);

        godot_print!("item index is: {}", index);
        godot_print!("item collected");
    }

    #[func]
    fn open_close(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    fn tick_attack_cooldown(&mut self, delta: f64) {
        if !self.can_slash {
            self.slash_timer += delta;
            if self.slash_timer >= ATTACK_COOLDOWN {
                self.can_slash = true;
                self.slash_timer = 0.0;
            }
        }
    }

    fn tick_hunger(&mut self, delta: f64) {
        let is_moving = self.base_mut().get_velocity().length() > 1.0;
        let drain_rate = if is_moving { 6.0 } else { 12.0 };

        self.hunger_drain_timer += delta;
        if self.hunger_drain_timer >= drain_rate {
            self.hunger_drain_timer = 0.0;
            if self.hunger > 0 {
                self.hunger -= 1;
                godot_print!("Hunger: {}", self.hunger);
            }
        }

        if self.hunger >= 18 && self.health < MAX_HEALTH {
            self.regen_timer += delta;
            if self.regen_timer >= 4.0 {
                self.regen_timer = 0.0;
                self.heal(1);
                godot_print!("Regenerated 1 HP, health: {}", self.health);
            }
        } else {
            self.regen_timer = 0.0;
        }

        if self.hunger == 0 && self.health > 1 {
            self.starvation_timer += delta;
            if self.starvation_timer >= 4.0 {
                self.starvation_timer = 0.0;
                self.take_damage(1);
                godot_print!("Starving! Health: {}", self.health);
            }
        } else {
            self.starvation_timer = 0.0;
        }
    }

    fn tick_confused(&mut self, delta: f64) {
        if !self.confused {
            return;
        }
        self.confused_timer += delta;
        if self.confused_timer >= 0.0 {
            self.confused = false;
            self.confused_timer = 0.0;
            godot_print!("Confused wore off.");
        }
    }

    #[func]
    pub fn apply_confused(&mut self, duration: f64) {
        if self.blessed {
            return;
        }
        self.confused = true;
        self.confused_timer = -duration;
        godot_print!("Confused applied for {}s!", duration);
    }

    #[func]
    pub fn is_confused(&self) -> bool {
        self.confused
    }

    fn tick_arrested(&mut self, delta: f64) {
        if !self.arrested {
            return;
        }
        self.arrested_timer += delta;
        if self.arrested_timer >= 0.0 {
            self.arrested = false;
            self.arrested_timer = 0.0;
            godot_print!("Arrest released.");
        }
    }

    #[func]
    pub fn apply_arrested(&mut self, duration: f64) {
        if self.blessed {
            return;
        }
        self.arrested = true;
        self.arrested_timer = -duration;
        godot_print!("Player arrested for {}s!", duration);
    }

    #[func]
    pub fn is_arrested(&self) -> bool {
        self.arrested
    }

    fn tick_indebted(&mut self, delta: f64) {
        if !self.indebted {
            return;
        }
        self.indebted_timer += delta;
        if self.indebted_timer >= INDEBTED_DURATION {
            self.indebted = false;
            self.indebted_timer = 0.0;
            godot_print!("Indebted debuff wore off.");
        }
    }

    fn effective_attack_damage(&self) -> i32 {
        if self.indebted {
            (ATTACK_DAMAGE - INDEBTED_ATTACK_REDUCTION).max(1)
        } else {
            ATTACK_DAMAGE
        }
    }

    fn attack(&mut self) {
        let damage = self.effective_attack_damage();
        let attack_area = self.attack_area.clone();

        for body in attack_area.get_overlapping_bodies().iter_shared() {
            if let Ok(mut crocodile) = body
                .clone()
                .try_cast::<crate::mobs::hostile::crocodile::Crocodile>()
            {
                crocodile.bind_mut().take_damage(damage);
            } else if let Ok(mut troll) = body
                .clone()
                .try_cast::<crate::mobs::hostile::troll::Troll>()
            {
                troll.bind_mut().take_damage(damage);
            } else if let Ok(mut order_force) =
                body.clone()
                    .try_cast::<crate::mobs::hostile::order_force::OrderForce>()
            {
                order_force.bind_mut().take_damage(damage);
            } else if let Ok(mut thug) =
                body.clone()
                    .try_cast::<crate::mobs::hostile::commissioned_thug::CommissionedThug>()
            {
                thug.bind_mut().take_damage(damage);
            } else if let Ok(mut snatcher) = body
                .clone()
                .try_cast::<crate::mobs::hostile::snatcher::Snatcher>()
            {
                snatcher.bind_mut().take_damage(damage);
            } else if let Ok(mut broker) =
                body.clone()
                    .try_cast::<crate::mobs::hostile::corruption_broker::CorruptionBroker>()
            {
                broker.bind_mut().take_damage(damage);
            } else if let Ok(mut vessel) =
                body.try_cast::<crate::mobs::hostile::smuggler_vessel::SmuglerVessel>()
            {
                vessel.bind_mut().take_damage(damage);
            }
        }
    }

    #[func]
    pub fn apply_indebted(&mut self) {
        self.indebted = true;
        self.indebted_timer = 0.0;
        godot_print!(
            "Player is now Indebted. Attack reduced by {} for {}s.",
            INDEBTED_ATTACK_REDUCTION,
            INDEBTED_DURATION
        );
    }

    // pub fn notify(&mut self, text: &str) {
    //     self.base_mut()
    //         .emit_signal("message", &[Variant::from(GString::from(text))]);
    // }

    #[func]
    pub fn is_indebted(&self) -> bool {
        self.indebted
    }

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.health = health.clamp(0, MAX_HEALTH);
        self.heart_ui.bind_mut().set_heart_display(self.health);
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }

    #[func]
    pub fn get_hunger(&self) -> i32 {
        self.hunger
    }

    #[func]
    pub fn set_hunger(&mut self, hunger: i32) {
        self.hunger = hunger.clamp(0, 20);
    }

    #[func]
    pub fn feed(&mut self, amount: i32) {
        self.hunger = (self.hunger + amount).clamp(0, 20);
        godot_print!("Fed player, hunger: {}", self.hunger);
    }

    #[func]
    pub fn get_heart_ui(&self) -> Gd<Heart> {
        self.heart_ui.clone()
    }

    #[func]
    fn on_animation_finished(&mut self) {
        self.playing_oneshot = false;
        self.sprite.play_ex().name("default").done();
    }
}

#[godot_api(secondary)]
impl Rustplayer {
    #[func]
    pub fn set_in_sanctuary(&mut self, value: bool) {
        self.in_sanctuary = value;
    }

    #[func]
    pub fn is_in_sanctuary(&self) -> bool {
        self.in_sanctuary
    }

    #[func]
    pub fn get_piso(&self) -> i32 {
        self.piso
    }

    #[func]
    pub fn add_piso(&mut self, amount: i32) {
        self.piso = (self.piso + amount).max(0);
        let piso = self.piso;
        self.base_mut()
            .emit_signal("piso_changed", &[Variant::from(piso)]);
        godot_print!("Piso +{}. Total: {}", amount, piso);
    }

    #[func]
    pub fn spend_piso(&mut self, amount: i32) -> bool {
        if self.piso < amount {
            godot_print!("Not enough piso. Have {}, need {}.", self.piso, amount);
            return false;
        }
        self.piso -= amount;
        let piso = self.piso;
        self.base_mut()
            .emit_signal("piso_changed", &[Variant::from(piso)]);
        godot_print!("Piso -{}. Total: {}", amount, piso);
        true
    }

    #[func]
    pub fn set_piso(&mut self, amount: i32) {
        self.piso = amount.max(0);
        let piso = self.piso;
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("piso_changed")),
                Variant::from(piso),
            ],
        );
    }

    fn tick_blessed(&mut self, delta: f64) {
        if !self.blessed {
            return;
        }
        self.blessed_timer -= delta;
        if self.blessed_timer <= 0.0 {
            self.blessed = false;
            self.blessed_timer = 0.0;
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from("Blessing wore off.")),
                ],
            );
        }
    }

    fn tick_speed_bonus(&mut self, delta: f64) {
        if self.speed_bonus == 0.0 {
            return;
        }
        self.speed_bonus_timer -= delta;
        if self.speed_bonus_timer <= 0.0 {
            self.speed_bonus = 0.0;
            self.speed_bonus_timer = 0.0;
            let mut event_bus = get_autoload_by_name::<Node>("EventBus");
            event_bus.call(
                "emit_signal",
                &[
                    Variant::from(GString::from("message")),
                    Variant::from(GString::from("Speed boost wore off.")),
                ],
            );
        }
    }

    #[func]
    pub fn apply_blessing(&mut self, duration: f64) {
        self.blessed = true;
        self.blessed_timer = duration;
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from("You are blessed. Debuffs blocked.")),
            ],
        );
    }

    #[func]
    pub fn is_blessed(&self) -> bool {
        self.blessed
    }

    #[func]
    pub fn apply_speed_bonus(&mut self, bonus: f32, duration: f64) {
        self.speed_bonus = bonus;
        self.speed_bonus_timer = duration;
        let mut event_bus = get_autoload_by_name::<Node>("EventBus");
        event_bus.call(
            "emit_signal",
            &[
                Variant::from(GString::from("message")),
                Variant::from(GString::from("Speed boosted!")),
            ],
        );
    }
}
