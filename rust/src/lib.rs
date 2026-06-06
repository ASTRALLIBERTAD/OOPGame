mod armor_system;
mod biome;
pub mod entity;
mod example;
pub mod heart;
pub mod heart_display;
mod inv_slot;
mod inventory;
mod item_collectibles;
mod item_slot;
mod main_node;
pub mod mobs;
mod multiplayer_scene;
mod node_manager;
mod pet;
mod rustplayer;
mod save_manager_rusts;
mod terrain;
mod world;

use godot::prelude::*;

pub struct RustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustExtension {}
