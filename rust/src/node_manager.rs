use godot::classes::{INode, Node};
use godot::prelude::*;

use crate::rustplayer::Rustplayer;
use crate::terrain::Terrain1;
use crate::world::Node2dRust;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct NodeManager {
    base: Base<Node>,
    terrain: Gd<Terrain1>,
    world: Gd<Node2dRust>,
    player: Option<Gd<Rustplayer>>,
}

#[godot_api]
impl INode for NodeManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            terrain: Terrain1::new_alloc(),
            world: Node2dRust::new_alloc(),
            player: None,
        }
    }
}

#[godot_api]
impl NodeManager {
    #[func]
    pub fn register_terrain(&mut self, terrain: Gd<Terrain1>) {
        self.terrain = terrain;
    }

    pub fn get_terrain(&mut self) -> Gd<Terrain1> {
        self.terrain.clone()
    }

    #[func]
    pub fn register_world(&mut self, world: Gd<Node2dRust>) {
        self.world = world;
    }

    pub fn get_world(&mut self) -> Gd<Node2dRust> {
        self.world.clone()
    }

    #[func]
    pub fn register_player(&mut self, player: Gd<Rustplayer>) {
        self.player = Some(player);
    }

    #[func]
    pub fn get_player(&mut self) -> Option<Gd<Rustplayer>> {
        self.player.clone()
    }
}
