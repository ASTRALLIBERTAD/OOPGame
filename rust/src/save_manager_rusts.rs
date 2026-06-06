use std::env::consts::OS;

use crate::node_manager::NodeManager;
use crate::rustplayer::Rustplayer;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use godot::classes::file_access::ModeFlags;
use godot::classes::{DirAccess, FileAccess, Node, Time};
use godot::prelude::*;
use godot::tools::get_autoload_by_name;
use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};

const PLAYER_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("player_data");
const CONFIG_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("config_data");

const CIPHER_KEY: &[u8; 32] = b"archipelago-chronicles-key-32byt";
const NONCE_BYTES: &[u8; 12] = b"ac-nonce-12b";

fn encrypt(plain: &[u8]) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(CIPHER_KEY));
    let nonce = Nonce::from_slice(NONCE_BYTES);
    cipher.encrypt(nonce, plain)
}

fn decrypt(cipher_text: &[u8]) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(CIPHER_KEY));
    let nonce = Nonce::from_slice(NONCE_BYTES);
    cipher.decrypt(nonce, cipher_text)
}

#[derive(Serialize, Deserialize)]
struct PlayerData {
    position_x: f32,
    position_y: f32,
    health: i32,
}

#[derive(Serialize, Deserialize)]
struct SaveGameInfo {
    #[serde(rename = "dateTime")]
    date_time: f64,
    #[serde(rename = "imgPath")]
    img_path: String,
    name: String,
    seed: i32,
}

#[derive(Serialize, Deserialize, Clone)]
struct ConfigSettings {
    #[serde(rename = "playerName")]
    player_name: String,
    volume: f32,
}

#[derive(GodotClass)]
#[class(base = Node, init)]
struct SaveManagerRust {
    #[base]
    base: Base<Node>,

    #[var(get, set)]
    current_world_name: StringName,

    #[export]
    #[var(get = get_seed, set = set_seed)]
    world_seed: i32,

    player_health: i32,
}

impl SaveManagerRust {
    fn base_path(&self) -> String {
        let baser: &str = match OS {
            "windows" => {
                godot_print!("windows");
                "user://"
            }
            "android" => {
                godot_print!("android");
                "/storage/emulated/0/Android/data/com.oopgame.project/files/"
            }
            _ => {
                godot_print!("linux");
                "user://"
            }
        };
        godot_print!("{}", baser);
        baser.to_string()
    }

    fn open_db(&self, world_name: &str) -> Option<Database> {
        let base = self.base_path();
        let games_dir = format!("{}/games", base);
        let world_dir = format!("{}/games/{}", base, world_name);
        let db_vpath = format!("{}/world.db", world_dir);

        if let Some(mut dir) = DirAccess::open(&base) {
            if !dir.dir_exists("games") {
                dir.make_dir("games");
            }
        }
        if let Some(mut dir) = DirAccess::open(&games_dir) {
            if !dir.dir_exists(world_name) {
                dir.make_dir(world_name);
            }
        }
        if let Some(mut dir) = DirAccess::open(&world_dir) {
            if !dir.dir_exists("chunk") {
                dir.make_dir("chunk");
            }
        }

        let db_path = godot::classes::ProjectSettings::singleton()
            .globalize_path(&db_vpath)
            .to_string();

        match Database::create(&db_path) {
            Ok(db) => Some(db),
            Err(e) => {
                godot_error!("Failed to open/create redb at {}: {}", db_path, e);
                None
            }
        }
    }

    fn get_player(&mut self) -> Option<Gd<Rustplayer>> {
        self.base_mut()
            .get_tree()
            .get_nodes_in_group("player")
            .iter_shared()
            .find_map(|node| node.try_cast::<Rustplayer>().ok())
    }

    fn write_encrypted<T: Serialize>(
        db: &Database,
        table_def: TableDefinition<&str, &[u8]>,
        key: &str,
        value: &T,
    ) -> bool {
        let raw = match bincode::serialize(value) {
            Ok(v) => v,
            Err(e) => {
                godot_error!("bincode serialise failed: {}", e);
                return false;
            }
        };

        let encrypted = match encrypt(&raw) {
            Ok(v) => v,
            Err(e) => {
                godot_error!("encryption failed: {}", e);
                return false;
            }
        };

        let write_txn = match db.begin_write() {
            Ok(t) => t,
            Err(e) => {
                godot_error!("redb write txn failed: {}", e);
                return false;
            }
        };

        {
            let mut table = match write_txn.open_table(table_def) {
                Ok(t) => t,
                Err(e) => {
                    godot_error!("redb open table failed: {}", e);
                    return false;
                }
            };
            if let Err(e) = table.insert(key, encrypted.as_slice()) {
                godot_error!("redb insert failed: {}", e);
                return false;
            };
        }

        if let Err(e) = write_txn.commit() {
            godot_error!("redb commit failed: {}", e);
            return false;
        }

        true
    }

    fn read_encrypted<T: for<'de> Deserialize<'de>>(
        db: &Database,
        table_def: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> Option<T> {
        let read_txn = match db.begin_read() {
            Ok(t) => t,
            Err(e) => {
                godot_error!("redb read txn failed: {}", e);
                return None;
            }
        };

        let table = match read_txn.open_table(table_def) {
            Ok(t) => t,
            Err(e) => {
                godot_error!("redb open table (read) failed: {}", e);
                return None;
            }
        };

        let guard = match table.get(key) {
            Ok(Some(v)) => v,
            Ok(None) => {
                godot_error!("redb key '{}' not found", key);
                return None;
            }
            Err(e) => {
                godot_error!("redb get failed: {}", e);
                return None;
            }
        };

        let decrypted = match decrypt(guard.value()) {
            Ok(v) => v,
            Err(e) => {
                godot_error!("decryption failed: {}", e);
                return None;
            }
        };

        match bincode::deserialize::<T>(&decrypted) {
            Ok(v) => Some(v),
            Err(e) => {
                godot_error!("bincode deserialise failed: {}", e);
                None
            }
        }
    }

    fn real_path(&self, vpath: &str) -> String {
        godot::classes::ProjectSettings::singleton()
            .globalize_path(vpath)
            .to_string()
    }

    fn read_config(&self) -> Option<ConfigSettings> {
        let db_path = self.real_path(&format!("{}/config.db", self.base_path()));
        let db = match Database::open(&db_path) {
            Ok(d) => d,
            Err(e) => {
                godot_error!("Failed to open config.db for reading: {}", e);
                return None;
            }
        };
        Self::read_encrypted::<ConfigSettings>(&db, CONFIG_TABLE, "config")
    }
}

#[godot_api]
impl SaveManagerRust {
    #[func]
    fn get_os(&self) -> String {
        self.base_path()
    }

    #[func]
    fn save_game_rust(&mut self, name: String) {
        self.current_world_name = StringName::from(&name);
        let _ = self.open_db(&name);
        self.set_player_health(20);
    }

    #[func]
    fn save_player_pos(&mut self, name: String) {
        self.current_world_name = StringName::from(&name);
        godot_print!("Current world name: {}", self.current_world_name);

        let Some(mut player) = self.get_player() else {
            godot_error!("No player found, skipping save");
            return;
        };
        let position = player.get_global_position();
        let player_data = PlayerData {
            position_x: position.x,
            position_y: position.y,
            health: player.bind_mut().get_health(),
        };
        godot_print!("Saving health: {}", player_data.health);

        let Some(db) = self.open_db(&name) else {
            return;
        };
        if Self::write_encrypted(&db, PLAYER_TABLE, &name, &player_data) {
            godot_print!("Player data saved to redb (world: {})", name);
        }

        let mut autoload = get_autoload_by_name::<NodeManager>("GlobalNodeManager");
        let world = autoload.bind_mut().get_world();
        let mut terrain = autoload.bind_mut().get_terrain();
        let mut terrain_ref = terrain.bind_mut();

        let player_name = world.bind().player_node_names.clone();
        terrain_ref.set_player_node_names(player_name);

        let path = format!(
            "{}/games/{}/chunk",
            self.base_path(),
            self.current_world_name
        );
        terrain_ref.set_path(path);

        let dirty_chunks: Vec<_> = terrain_ref
            .get_chunk_cache()
            .iter()
            .filter_map(|(pos, chunk)| if chunk.changed { Some(*pos) } else { None })
            .collect();

        for pos in dirty_chunks {
            terrain_ref.save_chunk(pos);
        }
    }

    #[func]
    fn load_player_pos(&mut self, name: String) {
        let mut autoload = get_autoload_by_name::<NodeManager>("GlobalNodeManager");
        let world = autoload.bind_mut().get_world();
        let mut binding = autoload.bind_mut().get_terrain();
        let mut terrain_ref = binding.bind_mut();

        let player_name = world.bind().player_node_names.clone();
        terrain_ref.set_player_node_names(player_name);

        let path = format!(
            "{}/games/{}/chunk",
            self.base_path(),
            self.current_world_name
        );
        terrain_ref.set_path(path);

        let Some(db) = self.open_db(&name) else {
            return;
        };

        let Some(player_data) = Self::read_encrypted::<PlayerData>(&db, PLAYER_TABLE, &name) else {
            godot_error!("Failed to load player data from redb (world: {})", name);
            return;
        };

        let Some(mut player) = self.get_player() else {
            godot_error!("No player found, skipping load");
            return;
        };

        player.set_global_position(Vector2::new(player_data.position_x, player_data.position_y));
        player.bind_mut().set_health(player_data.health);

        godot_print!("Player position loaded (world: {})", name);
        godot_print!("Loaded health: {}", player_data.health);
    }

    #[func]
    fn load_game(&mut self, name: GString) {
        self.current_world_name = StringName::from(&name);
        self.load_player_pos(name.to_string());
    }

    #[func]
    fn rust_screenshot(&mut self) {
        let world_name = self.current_world_name.clone();
        self.save_player_pos(world_name.to_string());
        godot_print!("world name is: {}", world_name);
        let path = format!(
            "{}/games/{}/{}.png",
            self.base_path(),
            world_name,
            world_name
        );
        let screen_capture = self
            .base_mut()
            .get_viewport()
            .unwrap()
            .get_texture()
            .unwrap()
            .get_image()
            .unwrap();
        screen_capture.save_png(&path);
    }

    #[func]
    fn auto_save(&mut self) {
        let world_name = self.current_world_name.clone();
        godot_print!("world name is: {}", world_name);
        if !world_name.is_empty() {
            self.save_player_pos(world_name.to_string());
        } else {
            godot_print!("no world");
        }
    }

    #[func]
    fn delete_save(&mut self, name: String) {
        let base_path = self.base_path();
        let folder = "games";
        let save_path = format!("{}/{}/{}", base_path, folder, name);

        if let Some(mut dir) = godot::classes::DirAccess::open(&save_path) {
            if dir.dir_exists(&save_path) {
                if self.delete_directory_recursive(&save_path) {
                    godot_print!("Save game '{}' deleted successfully.", name);
                } else {
                    godot_error!("Failed to delete save game '{}'.", name);
                }
            } else {
                godot_print!("Save game '{}' not found.", name);
            }
        } else {
            godot_print!("Save game '{}' not found (couldn't open dir).", name);
        }
    }

    fn delete_directory_recursive(&self, path: &str) -> bool {
        if let Some(mut dir) = godot::classes::DirAccess::open(path) {
            dir.list_dir_begin();
            loop {
                let entry = dir.get_next();
                if entry.is_empty() {
                    break;
                }
                if entry == "." || entry == ".." {
                    continue;
                }
                let full_path = format!("{}/{}", path, entry);
                if dir.current_is_dir() {
                    if !self.delete_directory_recursive(&full_path) {
                        return false;
                    }
                } else {
                    dir.remove(&full_path);
                }
            }
            dir.list_dir_end();

            if let Some(mut parent) = godot::classes::DirAccess::open(
                std::path::Path::new(path)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ) {
                parent.remove(path);
            }
            true
        } else {
            false
        }
    }

    #[func]
    fn save_world(&mut self) {
        let time = Time::singleton();
        let folder = "games";
        let save_path = format!(
            "{}/{}/{}/{}_saveGame.json",
            self.base_path(),
            folder,
            self.current_world_name,
            self.current_world_name
        );

        match FileAccess::open(&save_path, ModeFlags::WRITE) {
            Some(mut file) => {
                let info = SaveGameInfo {
                    date_time: time.get_unix_time_from_system(),
                    img_path: format!(
                        "{}/games/{}/{}.png",
                        self.base_path(),
                        self.current_world_name,
                        self.current_world_name
                    ),
                    name: self.current_world_name.to_string(),
                    seed: self.world_seed,
                };
                match serde_json::to_string(&info) {
                    Ok(json_string) => {
                        file.store_string(&json_string);
                        self.rust_screenshot();
                        godot_print!("Game info saved at {}", save_path);
                    }
                    Err(e) => godot_error!("Failed to serialise game info: {}", e),
                }
            }
            None => godot_error!("Failed to open save file at {}", save_path),
        }
    }

    #[func]
    fn save_config_json(&self, player_name: String, volume: f32) {
        let db_path = self.real_path(&format!("{}/config.db", self.base_path()));
        let settings = ConfigSettings {
            player_name,
            volume,
        };

        let db = match Database::create(&db_path) {
            Ok(d) => d,
            Err(e) => {
                godot_error!("Failed to open config.db: {}", e);
                return;
            }
        };

        if Self::write_encrypted(&db, CONFIG_TABLE, "config", &settings) {
            godot_print!("Config saved to {}", db_path);
        }
    }

    #[func]
    fn get_config_player_name(&self) -> String {
        self.read_config()
            .map(|s| s.player_name)
            .unwrap_or_else(|| "ASTRAL".to_string())
    }

    #[func]
    fn get_config_volume(&self) -> f32 {
        self.read_config().map(|s| s.volume).unwrap_or(50.0)
    }

    #[func]
    fn set_player_health(&mut self, health: i32) {
        if let Some(mut player) = self.get_player() {
            player.bind_mut().set_health(health);
        }
        self.player_health = health;
        godot_print!("Player health set to: {}", health);
    }

    #[func]
    fn set_current_world_name(&mut self, name: StringName) {
        self.current_world_name = name;
    }

    #[func]
    fn get_current_world_name(&self) -> StringName {
        self.current_world_name.clone()
    }

    #[func]
    fn set_seed(&mut self, seed: i32) {
        self.world_seed = seed;
    }

    #[func]
    fn get_seed(&self) -> i32 {
        self.world_seed
    }
}