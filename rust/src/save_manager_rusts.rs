use std::env::consts::OS;

use crate::node_manager::NodeManager;
use crate::rustplayer::Rustplayer;
use godot::classes::file_access::ModeFlags;
use godot::classes::{DirAccess, FileAccess, Node, Time};
use godot::prelude::*;
use godot::tools::get_autoload_by_name;
use serde::{Deserialize, Serialize};

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
pub struct SaveManagerRust {
    #[base]
    base: Base<Node>,

    #[var(get, set)]
    current_world_name: StringName,

    #[export]
    #[var(get = get_seed, set = set_seed)]
    world_seed: i32,

    player_health: i32,
}

#[godot_api]
impl SaveManagerRust {
    #[func]
    fn get_os(&self) -> String {
        let mut baser: &str = "";
        if OS == "windows" {
            baser = "user://";
            godot_print!("windows");
        } else if OS == "android" {
            baser = "/storage/emulated/0/Android/data/com.oopgame.project/files/";
            godot_print!("android");
        } else if OS == "linux" {
            baser = "user://";
            godot_print!("linux");
        }
        godot_print!("{}", baser);
        (&baser).to_string()
    }

    #[func]
    fn save_game_rust(&mut self, name: String) {
        self.current_world_name = StringName::from(&name);

        let base_path = &self.get_os();
        let folder = "games";
        let file_saver = format!("{}/{}", base_path, folder);
        let games_path = format!("{}/{}/{}", base_path, folder, name);

        let mut dir = DirAccess::open(base_path).expect("ok");

        if !dir.dir_exists(folder) {
            dir.make_dir(folder);
        }

        dir = DirAccess::open(&file_saver).expect("failed to open /games");

        if !dir.dir_exists(&name) {
            dir.make_dir(&name);
        }

        dir = DirAccess::open(&games_path).expect("failed to open world dir");

        if !dir.dir_exists("chunk") {
            dir.make_dir("chunk");
        }

        self.set_player_health(20);
    }

    #[func]
    fn save_player_pos(&mut self, name: String) {
        self.current_world_name = StringName::from(&name);
        godot_print!("Current world name: {}", self.current_world_name);

        let base_path = self.get_os();
        let folder = "games";
        let save_path = format!("{}/{}/{}/{}.dat", base_path, folder, name, name);

        match FileAccess::open(&save_path, ModeFlags::WRITE) {
            Some(mut file) => {
                let Some(mut player) = self.get_player() else {
                    godot_error!("No player found, skipping save");
                    return;
                };
                let position = player.get_global_position();
                let player_position = PlayerData {
                    position_x: position.x,
                    position_y: position.y,
                    health: player.bind_mut().get_health(),
                };

                match bincode::serialize(&player_position) {
                    Ok(serialized_data) => {
                        if serialized_data.len() <= 1048576 {
                            let byte_array = PackedByteArray::from(serialized_data);
                            file.store_buffer(&byte_array);
                            godot_print!("Game saved successfully at {}", save_path);
                            godot_print!("Game saved health {}", player_position.health);
                        } else {
                            godot_error!("Serialized data exceeds the size limit!");
                        }
                    }
                    Err(e) => {
                        godot_error!("Failed to serialize player position: {}", e);
                    }
                }
            }
            None => {
                godot_error!("Failed to open save file at {}", save_path);
            }
        }

        let mut autoload = get_autoload_by_name::<NodeManager>("GlobalNodeManager");

        let world = autoload.bind_mut().get_world();

        let mut terrain = autoload.bind_mut().get_terrain();

        let mut terrain_ref = terrain.bind_mut();

        let player_name = world.bind().player_node_names.clone();
        terrain_ref.set_player_node_names(player_name);

        let path = format!("{}/games/{}/chunk", self.get_os(), self.current_world_name);
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

        let path = format!("{}/games/{}/chunk", self.get_os(), self.current_world_name);
        terrain_ref.set_path(path);

        let base_path = self.get_os();
        let folder = "games";
        let save_path = format!("{}/{}/{}/{}.dat", base_path, folder, name, name);
        let file = FileAccess::open(&save_path, ModeFlags::READ);

        if let Some(mut file_acss) = file {
            let file_length = file_acss.get_length();
            let data = file_acss.get_buffer(file_length as i64);

            let data_slice: &[u8] = data.as_slice();

            match bincode::deserialize::<PlayerData>(data_slice) {
                Ok(player_data) => {
                    let Some(mut player) = self.get_player() else {
                        godot_error!("No player found, skipping load");
                        return;
                    };
                    player.set_global_position(Vector2::new(
                        player_data.position_x,
                        player_data.position_y,
                    ));
                    player.bind_mut().set_health(player_data.health);
                    godot_print!("Player position loaded successfully from {}", save_path);
                    godot_print!("saved heart {}", player_data.health);
                }
                Err(_) => {
                    godot_error!("Failed to deserialize player position from file");
                }
            }
        } else {
            godot_error!("Failed to open file for loading at {}", save_path);
        }
    }

    fn get_player(&mut self) -> Option<Gd<Rustplayer>> {
        self.base_mut()
            .get_tree()
            .get_nodes_in_group("player")
            .iter_shared()
            .find_map(|node| node.try_cast::<Rustplayer>().ok())
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
        let path = format!("{}/games/{}/{}.png", self.get_os(), world_name, world_name);
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
        let base_path = self.get_os();
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
            self.get_os(),
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
                        self.get_os(),
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
                        godot_print!("Game info saved successfully at {}", save_path);
                    }
                    Err(e) => {
                        godot_error!("Failed to serialize game info: {}", e);
                    }
                }
            }
            None => {
                godot_error!("Failed to open save file at {}", save_path);
            }
        }
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

    #[func]
    fn save_config_json(&self, player_name: String, volume: f32) {
        let base_path = self.get_os();
        let config_path = format!("{}/config.json", base_path);

        let settings = ConfigSettings {
            player_name,
            volume,
        };

        match FileAccess::open(&config_path, ModeFlags::WRITE) {
            Some(mut file) => match serde_json::to_string(&settings) {
                Ok(json_string) => {
                    file.store_string(&json_string);
                    godot_print!("Config settings saved successfully at {}", config_path);
                }
                Err(e) => {
                    godot_error!("Failed to serialize config json: {}", e);
                }
            },
            None => {
                godot_error!("Failed to create config file at {}", config_path);
            }
        }
    }

    #[func]
    pub fn get_config_player_name(&self) -> String {
        let base_path = self.get_os();
        let config_path = format!("{}/config.json", base_path);

        if let Some(file) = FileAccess::open(&config_path, ModeFlags::READ) {
            let json_string = file.get_as_text().to_string();
            if let Ok(settings) = serde_json::from_str::<ConfigSettings>(&json_string) {
                return settings.player_name;
            }
        }
        "ASTRAL".to_string()
    }

    #[func]
    pub fn get_config_volume(&self) -> f32 {
        let base_path = self.get_os();
        let config_path = format!("{}/config.json", base_path);

        if let Some(file) = FileAccess::open(&config_path, ModeFlags::READ) {
            let json_string = file.get_as_text().to_string();
            if let Ok(settings) = serde_json::from_str::<ConfigSettings>(&json_string) {
                return settings.volume;
            }
        }
        50.0 // Default fallback value
    }
}
