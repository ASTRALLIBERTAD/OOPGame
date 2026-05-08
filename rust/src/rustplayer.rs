use godot::classes::{
    AnimatedSprite2D, Area2D, Camera2D, CharacterBody2D, Control, ICharacterBody2D, Input, Label,
};
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::tools::get_autoload_by_name;
use std::str::FromStr;

use crate::entity::Entity;
use crate::heart::Heart;
use crate::inv_slot::InvSlot;
use crate::inventory::Inventory;
use crate::item_collectibles::Collectibles;
use crate::node_manager::NodeManager;

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

    // Optimization: Track last chunk position
    last_chunk_pos: Vector2i,
    last_update_time: f64,

    #[export]
    item_right: OnEditor<Gd<InvSlot>>,

    // slash_time: f64,
    can_slash: bool,

    #[export]
    attack_area: OnEditor<Gd<Area2D>>,
}

#[godot_api]
impl ICharacterBody2D for Rustplayer {
    fn init(base: Base<CharacterBody2D>) -> Self {
        Self {
            base,
            sprite: OnEditor::default(),
            coords: OnEditor::default(),
            inv: OnEditor::default(),
            item_slot: OnEditor::default(),
            is_open: false,
            heart_ui: OnEditor::default(),
            health: i32::default(),
            camera: OnEditor::default(),
            target_position: Vector2::default(),
            id: i32::default(),
            last_chunk_pos: Vector2i::new(i32::MAX, i32::MAX), // Invalid initial value
            last_update_time: 0.0,

            item_right: OnEditor::default(),
            // slash_time: 0.2,
            can_slash: true,
            attack_area: OnEditor::default(),
        }
    }

    fn ready(&mut self) {
        let pid = self.base_mut().get_multiplayer_authority();
        self.id = pid;

        godot_print!("Player ID is : {}", self.id);
        let is_authority = self.base_mut().is_multiplayer_authority();

        if !is_authority {
            self.camera.make_current();
        }
    }

    fn process(&mut self, delta: f64) {
        if self.base_mut().is_multiplayer_authority() {
            let speed: f32 = 100.0;
            let input = Input::singleton();

            let direction = Input::get_vector(
                &input,
                &StringName::from_str("left").unwrap(),
                &StringName::from_str("right").unwrap(),
                &StringName::from_str("up").unwrap(),
                &StringName::from_str("down").unwrap(),
            );

            let velocity = direction * speed;

            if input.is_action_just_pressed(&StringName::from_str("left").unwrap()) {
                self.sprite.set_flip_h(true);
                self.heart_ui.bind_mut().set_heart_display(1);
            }
            if input.is_action_just_pressed(&StringName::from_str("right").unwrap()) {
                self.sprite.set_flip_h(false);
                self.heart_ui.bind_mut().set_heart_display(-10);
            }

            self.base_mut().set_velocity(velocity);
            self.base_mut().move_and_slide();

            // CRITICAL FIX: Only update terrain when chunk changes or periodically
            self.update_terrain_if_needed(delta);

            // Update coordinates display
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

            // attack handling
            if input.is_action_just_pressed("attack") && self.can_slash {
                self.can_slash = false;
                self.attack();
                godot_print!("Player {} performed an attack!", self.id);
            }
        } else {
            let pos = self.target_position;
            let smooth_position = self
                .base_mut()
                .get_global_position()
                .lerp(pos, 10.0 * delta as f32);
            self.base_mut().set_global_position(smooth_position);

            // CRITICAL FIX: Remote players also need chunk updates but less frequently
            self.last_update_time += delta;
            if self.last_update_time >= 0.5 {
                // Update every 0.5 seconds for remote players
                self.update_terrain_if_needed(delta);
                self.last_update_time = 0.0;
            }
        }

        // let mut loader = load::<Inventory>("res://Collectibles/items/inventory.res");

        // self.base_mut().get_node_and_resource("res://Collectibles/items/inventory.res");
        // let name = loader.bind_mut().get_slots().get(0).unwrap().get_name();
        // self.item_right.set_name(name.to_godot());

        // godot_print!("The item name in index 0 is: {}", name)
    }
}
impl Entity for Rustplayer {
    fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        todo!("Implement this across the file")
    }
    fn heal(&mut self, amount: i32) {
        self.health += amount;
        todo!("Implement this across the file, refator the heal function at heart its not needed")
    }
    fn is_alive(&self) -> bool {
        self.health >= 0
    }
}
#[godot_api]
impl Rustplayer {
    #[rpc(unreliable, any_peer)]
    fn update_position(&mut self, pos: Vector2) {
        self.target_position = pos;
    }

    #[func]
    pub fn tester(&mut self, amount: i32) {
        godot_print!("connected and the amount is : {}", amount);
    }

    // OPTIMIZATION: Separate coordinate calculation from terrain updates
    fn get_player_cord_for_display(&mut self) -> Vector2 {
        let mut scene = get_autoload_by_name::<NodeManager>("GlobalNodeManager");
        let scene = scene.bind_mut().get_terrain();
        let local_position = self.base_mut().get_global_position();
        let cord = scene.local_to_map(local_position);
        scene.to_local(Vector2::new(cord.x as f32, cord.y as f32))
    }

    // CRITICAL OPTIMIZATION: Only update terrain when player moves to a new chunk
    fn update_terrain_if_needed(&mut self, _delta: f64) {
        let mut scene = get_autoload_by_name::<NodeManager>("GlobalNodeManager");
        let mut scene = scene.bind_mut().get_terrain();

        let pos = self.base_mut().get_global_position();
        let current_chunk = scene.local_to_map(pos);

        // Only update if we moved to a different chunk
        if current_chunk != self.last_chunk_pos {
            let id = self.base_mut().get_multiplayer_authority();
            scene.bind_mut().update_player_position(id, current_chunk);
            self.last_chunk_pos = current_chunk;

            // Optional: Debug logging
            // godot_print!("Player {} moved to chunk ({}, {})", id, current_chunk.x, current_chunk.y);
        }
    }

    // Original function kept for compatibility but optimized
    // fn player_cord(&mut self) -> Vector2 {
    //     self.get_player_cord_for_display()
    // }

    fn open(&mut self) {
        self.is_open = true;
        self.item_slot.set_visible(true);
    }

    fn close(&mut self) {
        self.is_open = false;
        self.item_slot.set_visible(false);
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

    #[func]
    pub fn set_health(&mut self, health: i32) {
        self.heart_ui.bind_mut().set_heart_display(health);
        self.health = health;
    }

    #[func]
    pub fn get_health(&self) -> i32 {
        self.health
    }
    //
    // pub fn player_hp(&mut self, num: i32) {
    //     self.heart_ui.bind_mut().heal(num);
    //     self.health = num;
    // }

    fn attack(&mut self) {
        self.attack_area.set_monitoring(true);
        self.attack_area.set_monitorable(true);

        let mut attack_area = self.attack_area.clone();
        let base = self.base().clone();
        godot::task::spawn(async move {
            godot_print!("starting task!");
            let timer = base.get_tree().create_timer(0.5);
            Signal::from_object_signal(&timer, "timeout")
                .to_future::<()>()
                .await;

            for body in attack_area.get_overlapping_bodies().iter_shared() {
                godot_print!("Found overlapping body during attack: {}", body);
                if body.is_in_group("enemy") {
                    godot_print!("Hit an enemy!");
                }
            }
            godot_print!("wait ended!");

            attack_area.set_monitoring(false);
            attack_area.set_monitorable(false);
        });
    }

    #[func]
    pub fn get_heart_ui(&self) -> Gd<Heart> {
        self.heart_ui.clone()
    }
}
