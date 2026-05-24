use crate::multiplayer::MultiPlayerRust;
use crate::rustplayer::Rustplayer;
use godot::classes::{FastNoiseLite, ITileMapLayer, InputEvent, TileMapLayer};
use godot::global::randi;
use godot::obj::WithBaseField;
use godot::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(GodotClass)]
#[class(base=TileMapLayer)]
pub struct ExampleTerrain1 {
    #[base]
    pub base: Base<TileMapLayer>,
    pub moisture: Gd<FastNoiseLite>,
    pub temperature: Gd<FastNoiseLite>,
    pub altitude: Gd<FastNoiseLite>,
    // Track both chunks and current positions
    pub player_chunks: HashMap<i32, HashSet<Vector2i>>,
    pub player_positions: HashMap<i32, Vector2i>,
    #[export]
    pub seedser: i32,
}

#[godot_api]
impl ITileMapLayer for ExampleTerrain1 {
    fn init(base: Base<TileMapLayer>) -> Self {
        Self {
            base,
            moisture: FastNoiseLite::new_gd(),
            temperature: FastNoiseLite::new_gd(),
            altitude: FastNoiseLite::new_gd(),
            player_chunks: HashMap::new(),
            player_positions: HashMap::new(),
            seedser: i32::default(),
        }
    }

    fn ready(&mut self) {
        self.base_mut()
            .get_tree()
            .get_root()
            .unwrap()
            .get_node_as::<Rustplayer>("/root/main/World/PLAYERS");
        self.moisture.set_seed(randi() as i32);
        self.temperature.set_seed(randi() as i32);
        self.altitude.set_frequency(0.01);
    }

    fn process(&mut self, _delta: f64) {
        self.altitude.set_seed(self.seedser);

        // Update main player
        let label = self
            .base_mut()
            .get_tree()
            .get_root()
            .unwrap()
            .get_node_as::<Rustplayer>("/root/main/World/PLAYERS");
        let ypo = label.get_position();
        let sls = self.base_mut().local_to_map(ypo);

        // Update main player position (using ID 0)
        self.player_positions.insert(0, sls);
        self.generate_chunk_for_player(0, sls);
        self.unload_distant_chunks_for_player(0, sls);

        let tree = self.base_mut().get_tree();
        let root = tree.get_root().unwrap();
        let multiplayer = tree.get_multiplayer().unwrap();
        let peers = multiplayer.get_peers();

        if multiplayer.is_server() {
            for i in peers.to_vec() {
                let pyr = format!("/root/main/World/{}", i);
                let y = root.get_node_as::<MultiPlayerRust>(&pyr);
                if y.is_instance_valid() {
                    let r = y.get_global_position();
                    let f = self.base_mut().local_to_map(r);
                    // Update multiplayer position
                    self.player_positions.insert(i, f);
                    self.generate_chunk_for_player(i, f);
                    self.unload_distant_chunks_for_player(i, f);
                    godot_print!("Player {} is valid", i);
                } else {
                    // Remove disconnected player data
                    self.player_positions.remove(&i);
                    self.player_chunks.remove(&i);
                    godot_print!("Player {} is not valid", i);
                }
            }
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("click") {
            let k = self.base_mut().get_global_mouse_position();
            let l = self.base_mut().local_to_map(k);
            self.base_mut()
                .set_cell_ex(l)
                .source_id(1)
                .atlas_coords(Vector2i::new(1, 0))
                .done();
        }
    }
}
const CHUNK_SIZE: i32 = 32;

#[godot_api]
impl ExampleTerrain1 {
    /// Converts tile map coordinates to chunk coordinates
    fn get_chunk_coord(&self, pos: Vector2i) -> Vector2i {
        Vector2i::new(pos.x.div_euclid(CHUNK_SIZE), pos.y.div_euclid(CHUNK_SIZE))
    }

    /// Generate a full chunk
    fn generate_chunk(&mut self, chunk_pos: Vector2i) {
        let mut tiles_to_set = Vec::new();

        let start_x = chunk_pos.x * CHUNK_SIZE;
        let start_y = chunk_pos.y * CHUNK_SIZE;

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let position = Vector2i::new(start_x + x, start_y + y);

                let alt = self
                    .altitude
                    .get_noise_2d(position.x as f32, position.y as f32)
                    * 10.0;

                let coords = if alt < 0.1 {
                    Vector2i::new(0, 11) // water
                } else {
                    Vector2i::new(1, 0) // grass
                };

                tiles_to_set.push((position, coords));
            }
        }

        for (position, coords) in tiles_to_set {
            self.base_mut()
                .set_cell_ex(position)
                .source_id(1)
                .atlas_coords(coords)
                .done();
        }
    }

    fn generate_chunk_for_player(&mut self, player_id: i32, pos: Vector2i) {
        let center_chunk = self.get_chunk_coord(pos);
        let load_radius = 1; // 3x3

        // Step 1: collect new chunks to generate
        let mut new_chunks = Vec::new();
        if let Some(player_chunks) = self.player_chunks.get(&player_id) {
            for dx in -load_radius..=load_radius {
                for dy in -load_radius..=load_radius {
                    let chunk_pos = Vector2i::new(center_chunk.x + dx, center_chunk.y + dy);
                    if !player_chunks.contains(&chunk_pos) {
                        new_chunks.push(chunk_pos);
                    }
                }
            }
        } else {
            // First time: player has no chunks yet, so generate the full 3x3
            for dx in -load_radius..=load_radius {
                for dy in -load_radius..=load_radius {
                    new_chunks.push(Vector2i::new(center_chunk.x + dx, center_chunk.y + dy));
                }
            }
        }

        // Step 2: actually generate and then insert into HashSet
        for chunk_pos in new_chunks {
            self.generate_chunk(chunk_pos);

            // now safe to borrow mutably just for inserting
            self.player_chunks
                .entry(player_id)
                .or_default()
                .insert(chunk_pos);
        }
    }

    fn unload_distant_chunks_for_player(&mut self, player_id: i32, pos: Vector2i) {
        let player_chunk = self.get_chunk_coord(pos);
        let unload_distance = 2; // chunks around the player

        let chunks_to_unload: Vec<Vector2i> = self
            .player_chunks
            .get(&player_id)
            .map(|chunks| {
                chunks
                    .iter()
                    .filter(|&&chunk| {
                        let dx = (chunk.x - player_chunk.x).abs();
                        let dy = (chunk.y - player_chunk.y).abs();
                        dx > unload_distance || dy > unload_distance
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        // Actually clear and remove unloaded chunks
        for chunk in &chunks_to_unload {
            self.clear_chunk(*chunk); // remove tiles from map
        }
        if let Some(chunks) = self.player_chunks.get_mut(&player_id) {
            for chunk in &chunks_to_unload {
                chunks.remove(chunk); // remove from player's chunk set
            }
        }
    }

    fn clear_chunk(&mut self, chunk_pos: Vector2i) {
        let start_x = chunk_pos.x * CHUNK_SIZE;
        let start_y = chunk_pos.y * CHUNK_SIZE;

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let position = Vector2i::new(start_x + x, start_y + y);

                self.base_mut()
                    .set_cell_ex(position)
                    .source_id(-1)
                    .atlas_coords(Vector2i::new(-1, -1))
                    .alternative_tile(-1)
                    .done();
            }
        }
    }
}

