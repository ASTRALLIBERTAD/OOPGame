use crate::biome;
use crate::main_node::MainNode;
use godot::classes::DirAccess;
use godot::classes::{
    file_access::ModeFlags, FastNoiseLite, FileAccess, ITileMapLayer, InputEvent, TileMapLayer,
};
use godot::global::randi;
use godot::obj::WithBaseField;
use godot::prelude::*;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct V2i {
    x: i32,
    y: i32,
}

impl From<Vector2i> for V2i {
    #[inline]
    fn from(v: Vector2i) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<V2i> for Vector2i {
    #[inline]
    fn from(v: V2i) -> Self {
        Vector2i::new(v.x, v.y)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChunkData {
    tiles: Vec<V2i>,
    #[serde(skip)]
    pub changed: bool,
    #[serde(skip, default = "std::time::Instant::now")]
    last_accessed: std::time::Instant,
}

// #[derive(Serialize, Deserialize, Default)]
// pub struct WorldMeta {
//     pub chunks: HashSet<(i32, i32)>,
// }

const CHUNK_SIZE: i32 = 16;
const CHUNK_SIZE_USIZE: usize = CHUNK_SIZE as usize;
const CHUNK_TILES: usize = CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE;
const DEFAULT_MAX_CACHED_CHUNKS: usize = 1000;
const LOAD_RADIUS: i32 = 2;
const UNLOAD_DISTANCE: i32 = 4;
const CHUNK_CLEANUP_AGE_SECS: u64 = 600;
const CHUNK_UNLOAD_AGE_SECS: u64 = 300;

// Multiplayer optimizations
const MAX_CHUNKS_PER_FRAME: usize = 3; // Reduced from 4
const MAX_UNLOADS_PER_FRAME: usize = 2;
const MAX_SAVES_PER_FRAME: usize = 2; // Reduced from 3 (disk I/O is expensive)
const CLEANUP_INTERVAL_FRAMES: u32 = 600; // Doubled - cleanup less often
const UNLOAD_INTERVAL_FRAMES: u32 = 120; // Doubled - check unloads less often

impl ChunkData {
    #[inline]
    fn new() -> Self {
        Self {
            tiles: vec![V2i { x: -1, y: -1 }; CHUNK_TILES],
            changed: false,
            last_accessed: std::time::Instant::now(),
        }
    }

    #[inline(always)]
    const fn index(x: usize, y: usize) -> usize {
        y * CHUNK_SIZE_USIZE + x
    }

    #[inline]
    fn set(&mut self, x: usize, y: usize, val: V2i) {
        self.tiles[Self::index(x, y)] = val;
        self.changed = true;
        self.last_accessed = std::time::Instant::now();
    }

    #[inline]
    fn get(&self, x: usize, y: usize) -> V2i {
        self.tiles[Self::index(x, y)]
    }

    #[inline]
    fn touch(&mut self) {
        self.last_accessed = std::time::Instant::now();
    }
}

#[derive(GodotClass)]
#[class(base=TileMapLayer)]
pub struct Terrain1 {
    #[base]
    base: Base<TileMapLayer>,
    moisture: Gd<FastNoiseLite>,
    temperature: Gd<FastNoiseLite>,
    altitude: Gd<FastNoiseLite>,
    biome: Gd<FastNoiseLite>,
    player_chunks: HashMap<i32, HashSet<Vector2i>>,
    player_positions: HashMap<i32, Vector2i>,
    chunk_cache: HashMap<Vector2i, ChunkData>,
    loaded_chunks: HashSet<Vector2i>,
    path: String,
    #[export]
    #[var(get, set)]
    world_seed: i32,
    max_cached_chunks: usize,
    player_node_names: Array<GString>,

    // Multiplayer optimizations
    chunk_load_queue: VecDeque<Vector2i>,
    chunk_unload_queue: VecDeque<Vector2i>,
    chunk_save_queue: VecDeque<Vector2i>,
    frame_counter: u32,
    chunks_generated_this_frame: usize,
    chunks_unloaded_this_frame: usize,
    chunks_saved_this_frame: usize,
    all_players_chunks: HashSet<Vector2i>, // Shared chunk tracking
    corruption_tiles: HashSet<Vector2i>,
}

#[godot_api]
impl ITileMapLayer for Terrain1 {
    fn init(base: Base<TileMapLayer>) -> Self {
        Self {
            base,
            moisture: FastNoiseLite::new_gd(),
            temperature: FastNoiseLite::new_gd(),
            altitude: FastNoiseLite::new_gd(),
            biome: FastNoiseLite::new_gd(),
            player_chunks: HashMap::new(),
            player_positions: HashMap::new(),
            chunk_cache: HashMap::new(),
            loaded_chunks: HashSet::new(),
            path: String::default(),
            world_seed: i32::default(),
            max_cached_chunks: DEFAULT_MAX_CACHED_CHUNKS,
            player_node_names: Array::default(),
            chunk_load_queue: VecDeque::new(),
            chunk_unload_queue: VecDeque::new(),
            chunk_save_queue: VecDeque::new(),
            frame_counter: 0,
            chunks_generated_this_frame: 0,
            chunks_unloaded_this_frame: 0,
            chunks_saved_this_frame: 0,
            all_players_chunks: HashSet::new(),
            corruption_tiles: HashSet::new(),
        }
    }

    fn enter_tree(&mut self) {
        let callable = self.to_gd().callable("sync_seed");
        let mut main = self.base_mut().get_node_as::<MainNode>("/root/main");
        main.connect("seed_requested", &callable);
    }

    fn ready(&mut self) {
        self.altitude.set_seed(self.world_seed);
        self.moisture.set_seed(randi() as i32);
        self.temperature.set_seed(randi() as i32);
        self.altitude.set_frequency(0.01);
        // Biome noise: deterministic from world_seed, low frequency -> large regions.
        self.biome
            .set_seed(self.world_seed.wrapping_add(biome::BIOME_SEED_OFFSET));
        self.biome.set_frequency(biome::BIOME_FREQUENCY);
        godot_print!("Terrain1 ready with seed: {}", self.world_seed);
    }

    fn process(&mut self, _delta: f64) {
        // Don't process anything if no players
        if self.player_positions.is_empty() {
            return;
        }

        self.frame_counter += 1;
        self.chunks_generated_this_frame = 0;
        self.chunks_unloaded_this_frame = 0;
        self.chunks_saved_this_frame = 0;

        // Process chunk load queue with frame budget
        self.process_chunk_queue();

        // Process unload queue with frame budget
        self.process_unload_queue();

        // Process save queue with frame budget
        self.process_save_queue();

        // Periodic unloading (less frequent) - just queue chunks, don't process immediately
        if self.frame_counter.is_multiple_of(UNLOAD_INTERVAL_FRAMES) {
            self.queue_distant_chunks_for_unload();
        }

        // Periodic cleanup (even less frequent)
        if self.frame_counter.is_multiple_of(CLEANUP_INTERVAL_FRAMES) {
            self.cleanup_old_chunks();
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("click") {
            let k = self.base_mut().get_global_mouse_position();
            let l = self.base_mut().local_to_map(k);

            if self.corruption_tiles.contains(&l) {
                self.cleanse_corruption_tile(l);
            } else {
                let coords = Vector2i::new(1, 0);
                self.base_mut()
                    .set_cell_ex(l)
                    .source_id(1)
                    .atlas_coords(coords)
                    .done();

                let chunk_pos = Self::get_chunk_coord_static(l);
                let entry = self
                    .chunk_cache
                    .entry(chunk_pos)
                    .or_insert_with(ChunkData::new);

                let lx = l.x.rem_euclid(CHUNK_SIZE) as usize;
                let ly = l.y.rem_euclid(CHUNK_SIZE) as usize;
                entry.set(lx, ly, coords.into());
            }
        }
    }
}

#[godot_api]
impl Terrain1 {
    #[func]
    fn set_world_seed(&mut self, seed: i32) {
        self.world_seed = seed;
    }

    #[func]
    fn get_world_seed(&self) -> i32 {
        self.world_seed
    }
    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }
    pub fn set_player_node_names(&mut self, name: Array<GString>) {
        self.player_node_names = name;
    }

    pub fn get_chunk_cache(&self) -> HashMap<Vector2i, ChunkData> {
        self.chunk_cache.clone()
    }

    pub fn sync_seed(&mut self, seed: i32) {
        self.world_seed = seed;
        self.altitude.set_seed(self.world_seed);
        self.biome
            .set_seed(self.world_seed.wrapping_add(biome::BIOME_SEED_OFFSET));
        godot_print!("Terrain1 synced callable seed: {}", seed);
    }

    pub fn update_player_position(&mut self, id: i32, cord: Vector2i) {
        let new_chunk = Self::get_chunk_coord_static(cord);
        let moved_chunk = self
            .player_positions
            .get(&id)
            .map(|&old_pos| Self::get_chunk_coord_static(old_pos) != new_chunk)
            .unwrap_or(true);

        self.player_positions.insert(id, cord);

        if moved_chunk {
            self.queue_chunks_for_player(id, cord);
        }
    }

    // Queue chunks instead of generating immediately
    fn queue_chunks_for_player(&mut self, player_id: i32, pos: Vector2i) {
        let center_chunk = self.get_chunk_coord(pos);
        let mut new_chunks = Vec::new();

        for dx in -LOAD_RADIUS..=LOAD_RADIUS {
            for dy in -LOAD_RADIUS..=LOAD_RADIUS {
                let chunk_pos = Vector2i::new(center_chunk.x + dx, center_chunk.y + dy);

                // Add to all players tracking
                self.all_players_chunks.insert(chunk_pos);

                if !self.loaded_chunks.contains(&chunk_pos)
                    && !self.chunk_load_queue.contains(&chunk_pos)
                {
                    // Prioritize closer chunks
                    let distance = dx.abs() + dy.abs();
                    new_chunks.push((distance, chunk_pos));
                }
            }
        }

        // Sort by distance (closest first)
        new_chunks.sort_by_key(|(dist, _)| *dist);

        // Add to queue front (priority) or back based on distance
        for (dist, chunk_pos) in new_chunks {
            if dist <= 1 {
                self.chunk_load_queue.push_front(chunk_pos);
            } else {
                self.chunk_load_queue.push_back(chunk_pos);
            }
        }

        // Update player chunk tracking
        let player_chunks = self.player_chunks.entry(player_id).or_default();
        player_chunks.clear();
        for dx in -LOAD_RADIUS..=LOAD_RADIUS {
            for dy in -LOAD_RADIUS..=LOAD_RADIUS {
                player_chunks.insert(Vector2i::new(center_chunk.x + dx, center_chunk.y + dy));
            }
        }
    }

    // Process chunk queue with frame budget
    fn process_chunk_queue(&mut self) {
        while self.chunks_generated_this_frame < MAX_CHUNKS_PER_FRAME {
            let Some(chunk_pos) = self.chunk_load_queue.pop_front() else {
                break;
            };

            // Skip if already loaded (might have been loaded by another player)
            if self.loaded_chunks.contains(&chunk_pos) {
                continue;
            }

            self.generate_chunk_immediate(chunk_pos);
            self.chunks_generated_this_frame += 1;
        }
    }

    // Process unload queue with frame budget - CRITICAL for preventing lag spikes
    fn process_unload_queue(&mut self) {
        while self.chunks_unloaded_this_frame < MAX_UNLOADS_PER_FRAME {
            let Some(chunk_pos) = self.chunk_unload_queue.pop_front() else {
                break;
            };

            // Queue for saving if changed
            if let Some(chunk) = self.chunk_cache.get(&chunk_pos) {
                if chunk.changed {
                    self.chunk_save_queue.push_back(chunk_pos);
                }
            }

            // Clear the chunk from tilemap
            self.clear_chunk(chunk_pos);
            self.loaded_chunks.remove(&chunk_pos);
            self.all_players_chunks.remove(&chunk_pos);

            self.chunks_unloaded_this_frame += 1;
        }
    }

    // Process save queue with frame budget - spread disk I/O across frames
    fn process_save_queue(&mut self) {
        while self.chunks_saved_this_frame < MAX_SAVES_PER_FRAME {
            let Some(chunk_pos) = self.chunk_save_queue.pop_front() else {
                break;
            };

            self.save_chunk_immediate(chunk_pos);
            self.chunks_saved_this_frame += 1;
        }
    }

    // Optimized noise generation with pre-allocated buffer
    fn generate_noise_map(&self, chunk_pos: Vector2i) -> Vec<f32> {
        let mut noise_map = Vec::with_capacity(CHUNK_TILES);
        let start_x = chunk_pos.x * CHUNK_SIZE;
        let start_y = chunk_pos.y * CHUNK_SIZE;

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let nx = (start_x + x) as f32;
                let ny = (start_y + y) as f32;
                noise_map.push(self.altitude.get_noise_2d(nx, ny));
            }
        }
        noise_map
    }

    #[inline(always)]
    const fn get_chunk_coord_static(pos: Vector2i) -> Vector2i {
        Vector2i::new(pos.x.div_euclid(CHUNK_SIZE), pos.y.div_euclid(CHUNK_SIZE))
    }

    #[inline]
    fn get_chunk_coord(&self, pos: Vector2i) -> Vector2i {
        Self::get_chunk_coord_static(pos)
    }

    // Renamed to be explicit about immediate generation
    fn generate_chunk_immediate(&mut self, chunk_pos: Vector2i) {
        if self.loaded_chunks.contains(&chunk_pos) {
            if let Some(chunk) = self.chunk_cache.get_mut(&chunk_pos) {
                chunk.touch();
            }
            return;
        }

        // Try to load from disk first
        if self.load_chunk(chunk_pos) {
            self.loaded_chunks.insert(chunk_pos);
            return;
        }

        // Generate new chunk
        let mut chunk = ChunkData::new();
        let noise_map = self.generate_noise_map(chunk_pos);
        let start_x = chunk_pos.x * CHUNK_SIZE;
        let start_y = chunk_pos.y * CHUNK_SIZE;

        // Set tiles directly without batching for better performance
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let noise = noise_map[ChunkData::index(x as usize, y as usize)];
                let (source_id, coords) = if noise < 0.1 {
                    // Below the altitude threshold: water (unchanged).
                    (1, V2i { x: 0, y: 11 })
                } else {
                    // Land: pick this cell's Luzon biome from the biome noise.
                    let bnoise = self
                        .biome
                        .get_noise_2d((start_x + x) as f32, (start_y + y) as f32);
                    let (tx, ty) = biome::select_biome(bnoise).land_tile();
                    (biome::BIOME_SOURCE_ID, V2i { x: tx, y: ty })
                };

                chunk.tiles[ChunkData::index(x as usize, y as usize)] = coords;
                let tile_pos = Vector2i::new(start_x + x, start_y + y);

                // Direct tile setting - no batching overhead
                self.base_mut()
                    .set_cell_ex(tile_pos)
                    .source_id(source_id)
                    .atlas_coords(coords.into())
                    .done();
            }
        }

        chunk.changed = true;
        chunk.last_accessed = std::time::Instant::now();
        self.chunk_cache.insert(chunk_pos, chunk);
        self.loaded_chunks.insert(chunk_pos);
    }

    // Queue chunks for unloading instead of unloading immediately
    fn queue_distant_chunks_for_unload(&mut self) {
        if self.player_positions.is_empty() {
            return;
        }

        for &chunk_pos in &self.loaded_chunks {
            let mut keep_chunk = false;

            // Check against all active player positions
            for &player_pos in self.player_positions.values() {
                let player_chunk_pos = self.get_chunk_coord(player_pos);
                let dx = (chunk_pos.x - player_chunk_pos.x).abs();
                let dy = (chunk_pos.y - player_chunk_pos.y).abs();

                if dx <= UNLOAD_DISTANCE && dy <= UNLOAD_DISTANCE {
                    keep_chunk = true;
                    break;
                }
            }

            if !keep_chunk && !self.chunk_unload_queue.contains(&chunk_pos) {
                self.chunk_unload_queue.push_back(chunk_pos);
            }
        }
    }

    fn clear_chunk(&mut self, chunk_pos: Vector2i) {
        let start_x = chunk_pos.x * CHUNK_SIZE;
        let start_y = chunk_pos.y * CHUNK_SIZE;

        // Batch clear operations for better performance
        let empty_coords = Vector2i::new(-1, -1);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let position = Vector2i::new(start_x + x, start_y + y);
                self.base_mut()
                    .set_cell_ex(position)
                    .source_id(-1)
                    .atlas_coords(empty_coords)
                    .alternative_tile(-1)
                    .done();
            }
        }
    }

    // Renamed to indicate immediate save (used by save queue)
    fn save_chunk_immediate(&mut self, chunk_pos: Vector2i) {
        let Some(chunk) = self.chunk_cache.get_mut(&chunk_pos) else {
            return;
        };

        if !chunk.changed {
            return;
        }

        if !self.path.is_empty() {
            if let Some(mut dir) = DirAccess::open(&self.path) {
                if !dir.dir_exists(".") {
                    dir.make_dir_recursive(".");
                }
            } else if let Some(mut dir) = DirAccess::open("user://") {
                dir.make_dir_recursive(&self.path);
            }
        }

        let save_path = format!("{}/chunk_{}_{}.dat", self.path, chunk_pos.x, chunk_pos.y);

        if let Some(mut file) = FileAccess::open(&save_path, ModeFlags::WRITE) {
            if let Ok(serialized) = bincode::serialize(&*chunk) {
                let compressed = compress_prepend_size(&serialized);
                let buffer = PackedByteArray::from(compressed);
                file.store_buffer(&buffer);
                chunk.changed = false;
            }
        }

        // Remove from cache after saving if old and not loaded
        if !self.loaded_chunks.contains(&chunk_pos)
            && chunk.last_accessed.elapsed().as_secs() > CHUNK_UNLOAD_AGE_SECS
        {
            self.chunk_cache.remove(&chunk_pos);
        }
    }

    // Keep old function for manual saves
    pub fn save_chunk(&mut self, chunk_pos: Vector2i) {
        self.save_chunk_immediate(chunk_pos);
    }

    fn load_chunk(&mut self, chunk_pos: Vector2i) -> bool {
        let save_path = format!("{}/chunk_{}_{}.dat", self.path, chunk_pos.x, chunk_pos.y);

        let Some(mut file) = FileAccess::open(&save_path, ModeFlags::READ) else {
            return false;
        };

        let file_length = file.get_length();
        let buffer = file.get_buffer(file_length as i64);
        let Ok(decompressed) = decompress_size_prepended(buffer.as_slice()) else {
            return false;
        };

        let Ok(mut chunk) = bincode::deserialize::<ChunkData>(&decompressed) else {
            return false;
        };

        chunk.last_accessed = std::time::Instant::now();
        let start_x = chunk_pos.x * CHUNK_SIZE;
        let start_y = chunk_pos.y * CHUNK_SIZE;

        // Batch tile restoration
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let coords = chunk.get(x as usize, y as usize);
                if coords.x >= 0 {
                    let pos = Vector2i::new(start_x + x, start_y + y);
                    self.base_mut()
                        .set_cell_ex(pos)
                        .source_id(1)
                        .atlas_coords(coords.into())
                        .done();
                }
            }
        }

        self.chunk_cache.insert(chunk_pos, chunk);
        true
    }

    fn cleanup_old_chunks(&mut self) {
        if self.chunk_cache.len() <= self.max_cached_chunks {
            return;
        }

        let now = std::time::Instant::now();
        let mut old_chunks: Vec<_> = self
            .chunk_cache
            .iter()
            .filter(|(&pos, chunk)| {
                !chunk.changed
                    && !self.loaded_chunks.contains(&pos)
                    && !self.all_players_chunks.contains(&pos)
                    && now.duration_since(chunk.last_accessed).as_secs() > CHUNK_CLEANUP_AGE_SECS
            })
            .map(|(&pos, chunk)| (pos, chunk.last_accessed))
            .collect();

        old_chunks.sort_by_key(|(_, time)| *time);

        let remove_count = (self.chunk_cache.len() - self.max_cached_chunks).min(old_chunks.len());
        for (chunk_pos, _) in old_chunks.into_iter().take(remove_count) {
            self.chunk_cache.remove(&chunk_pos);
        }
    }

    #[func]
    fn set_save_path(&mut self, path: String) {
        self.path = path;
    }

    #[func]
    #[inline]
    fn get_loaded_chunk_count(&self) -> i32 {
        self.loaded_chunks.len() as i32
    }

    #[func]
    #[inline]
    fn get_cached_chunk_count(&self) -> i32 {
        self.chunk_cache.len() as i32
    }

    #[func]
    #[inline]
    fn get_queue_size(&self) -> i32 {
        self.chunk_load_queue.len() as i32
    }

    #[func]
    #[inline]
    fn get_unload_queue_size(&self) -> i32 {
        self.chunk_unload_queue.len() as i32
    }

    #[func]
    #[inline]
    fn get_save_queue_size(&self) -> i32 {
        self.chunk_save_queue.len() as i32
    }

    #[func]
    fn debug_chunk_info(&self, world_pos: Vector2i) -> String {
        let chunk_pos = self.get_chunk_coord(world_pos);
        let is_loaded = self.loaded_chunks.contains(&chunk_pos);
        let is_cached = self.chunk_cache.contains_key(&chunk_pos);
        let is_queued = self.chunk_load_queue.contains(&chunk_pos);

        format!(
            "World pos: ({}, {}), Chunk: ({}, {}), Loaded: {}, Cached: {}, Queued: {}, Load Q: {}, Unload Q: {}, Save Q: {}, Players: {}",
            world_pos.x, world_pos.y,
            chunk_pos.x, chunk_pos.y,
            is_loaded, is_cached, is_queued,
            self.chunk_load_queue.len(),
            self.chunk_unload_queue.len(),
            self.chunk_save_queue.len(),
            self.player_positions.len()
        )
    }

    #[func]
    fn force_generate_chunk_at(&mut self, world_pos: Vector2i) {
        let chunk_pos = self.get_chunk_coord(world_pos);
        self.loaded_chunks.remove(&chunk_pos);
        self.chunk_cache.remove(&chunk_pos);
        self.generate_chunk_immediate(chunk_pos);
    }

    #[func]
    fn force_save_all_chunks(&mut self) {
        let chunks_to_save: Vec<_> = self
            .chunk_cache
            .iter()
            .filter(|(_, chunk)| chunk.changed)
            .map(|(&pos, _)| pos)
            .collect();

        // Save immediately, bypassing queue for force save
        for chunk_pos in chunks_to_save {
            self.save_chunk_immediate(chunk_pos);
        }
    }

    #[func]
    fn set_performance_mode(&mut self, low_end: bool) {
        if low_end {
            self.max_cached_chunks = 300; // Further reduced from 500
            godot_print!("Performance mode enabled: max_cached_chunks = 300");
        } else {
            self.max_cached_chunks = DEFAULT_MAX_CACHED_CHUNKS;
            godot_print!("Performance mode disabled: max_cached_chunks = 1000");
        }
    }

    #[func]
    fn set_aggressive_performance_mode(&mut self) {
        self.max_cached_chunks = 200;
        godot_print!(
            "Aggressive performance mode enabled: Chunks will load slower but FPS should improve"
        );
    }

    #[func]
    pub fn spawn_corruption_tile(&mut self, tile_pos: Vector2i) {
        let corruption_coords = Vector2i::new(2, 0);
        self.base_mut()
            .set_cell_ex(tile_pos)
            .source_id(1)
            .atlas_coords(corruption_coords)
            .done();

        let chunk_pos = Self::get_chunk_coord_static(tile_pos);
        let entry = self
            .chunk_cache
            .entry(chunk_pos)
            .or_insert_with(ChunkData::new);
        let lx = tile_pos.x.rem_euclid(CHUNK_SIZE) as usize;
        let ly = tile_pos.y.rem_euclid(CHUNK_SIZE) as usize;
        entry.set(lx, ly, corruption_coords.into());

        self.corruption_tiles.insert(tile_pos);
        godot_print!(
            "Corruption tile spawned at ({}, {})",
            tile_pos.x,
            tile_pos.y
        );
    }

    #[func]
    pub fn cleanse_corruption_tile(&mut self, tile_pos: Vector2i) {
        if !self.corruption_tiles.contains(&tile_pos) {
            return;
        }

        let clean_coords = Vector2i::new(1, 0);
        self.base_mut()
            .set_cell_ex(tile_pos)
            .source_id(1)
            .atlas_coords(clean_coords)
            .done();

        let chunk_pos = Self::get_chunk_coord_static(tile_pos);
        let entry = self
            .chunk_cache
            .entry(chunk_pos)
            .or_insert_with(ChunkData::new);
        let lx = tile_pos.x.rem_euclid(CHUNK_SIZE) as usize;
        let ly = tile_pos.y.rem_euclid(CHUNK_SIZE) as usize;
        entry.set(lx, ly, clean_coords.into());

        self.corruption_tiles.remove(&tile_pos);
        godot_print!(
            "Corruption tile cleansed at ({}, {})",
            tile_pos.x,
            tile_pos.y
        );
    }

    #[func]
    pub fn get_corruption_tile_count(&self) -> i32 {
        self.corruption_tiles.len() as i32
    }

    #[func]
    fn flush_all_queues(&mut self) {
        godot_print!("Flushing all queues...");

        // Process all pending loads
        while !self.chunk_load_queue.is_empty() {
            if let Some(chunk_pos) = self.chunk_load_queue.pop_front() {
                if !self.loaded_chunks.contains(&chunk_pos) {
                    self.generate_chunk_immediate(chunk_pos);
                }
            }
        }

        // Process all pending unloads
        while !self.chunk_unload_queue.is_empty() {
            if let Some(chunk_pos) = self.chunk_unload_queue.pop_front() {
                if let Some(chunk) = self.chunk_cache.get(&chunk_pos) {
                    if chunk.changed {
                        self.chunk_save_queue.push_back(chunk_pos);
                    }
                }
                self.clear_chunk(chunk_pos);
                self.loaded_chunks.remove(&chunk_pos);
            }
        }

        // Process all pending saves
        while !self.chunk_save_queue.is_empty() {
            if let Some(chunk_pos) = self.chunk_save_queue.pop_front() {
                self.save_chunk_immediate(chunk_pos);
            }
        }

        godot_print!("All queues flushed!");
    }
}
