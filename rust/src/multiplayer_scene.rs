use godot::classes::{
    Button, ENetMultiplayerPeer, HBoxContainer, INode, Json, Label, Node, VBoxContainer,
};
use godot::prelude::*;
use local_ip_address::local_ip;
use std::net::UdpSocket;
const LISTEN_PORT: u16 = 8912;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct MultiplayerScene {
    base: Base<Node>,
    socket: Option<UdpSocket>,
    #[export]
    player: OnEditor<Gd<PackedScene>>,
    #[export]
    server_info: OnEditor<Gd<PackedScene>>,
}

#[godot_api]
impl INode for MultiplayerScene {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            socket: None,
            player: OnEditor::default(),
            server_info: OnEditor::default(),
        }
    }

    fn ready(&mut self) {
        self.set_up();
        let callable = self.base().callable("on_back_pressed");
        self.base_mut()
            .get_node_as::<Button>("CanvasLayer/back")
            .connect("pressed", &callable);
    }
    fn exit_tree(&mut self) {
        self.socket = None;
    }

    fn process(&mut self, _delta: f64) {
        let mut packets: Vec<(String, String)> = Vec::new(); // (ip, data)

        if let Some(socket) = &self.socket {
            let mut buf = [0u8; 1024];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((len, src_addr)) => {
                        godot_print!("RAW recv {} bytes from {}", len, src_addr);
                        let data = match std::str::from_utf8(&buf[..len]) {
                            Ok(s) => s.to_string(),
                            Err(_) => {
                                godot_error!("Invalid UTF-8");
                                continue;
                            }
                        };
                        packets.push((src_addr.ip().to_string(), data));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => {
                        godot_error!("Socket error: {}", e);
                        break;
                    }
                }
            }
        }

        for (ip, data) in packets {
            let serverip = GString::from(&ip);
            godot_print!("Packet from {}: {}", serverip, data);

            let mut json = Json::new_gd();
            if json.parse(&data) != godot::global::Error::OK {
                godot_error!("JSON parse failed: {}", data);
                continue;
            }

            let room_info = json.get_data();
            let Ok(dict) = room_info.try_to::<Dictionary<Variant, Variant>>() else {
                godot_error!(
                    "Not a dictionary — variant type: {:?}",
                    room_info.get_type()
                );
                continue;
            };

            let key = GString::from("name").to_variant();
            let name_str = dict
                .get(&key)
                .unwrap_or_else(|| Variant::from("Unnamed"))
                .to::<GString>();
            let mut vbox = self
                .base_mut()
                .get_node_as::<VBoxContainer>("CanvasLayer/Panel/VBoxContainer");

            let mut room_exists = false;
            for i in vbox.get_children().iter_shared() {
                if i.get_name().to_string() == name_str.to_string() {
                    if let Some(mut ip_label) = i.try_get_node_as::<Label>("Ip") {
                        ip_label.set_text(&serverip);
                    }
                    room_exists = true;
                    break;
                }
            }

            if !room_exists {
                let mut current_info = self.server_info.instantiate_as::<HBoxContainer>();
                current_info.set_name(&name_str.to_string());

                if let Some(mut ip_label) = current_info.try_get_node_as::<Label>("Ip") {
                    ip_label.set_text(&serverip);
                }
                if let Some(mut name_label) = current_info.try_get_node_as::<Label>("Name") {
                    name_label.set_text(&name_str);
                }

                vbox.add_child(&current_info);

                let callable = self.base_mut().callable("joinby_ip");
                current_info.connect("joinGame", &callable);

                godot_print!("Added new server: {}", name_str);
            }
        }
    }
}

#[godot_api]
impl MultiplayerScene {
    #[signal]
    fn join_game(ip: GString);

    fn get_broadcast_address(&self) -> String {
        match local_ip() {
            Ok(ip) => {
                // Convert x.x.x.x to x.x.x.255 (assumes /24 subnet)
                let mut octets = match ip {
                    std::net::IpAddr::V4(v4) => v4.octets(),
                    _ => return "255.255.255.255".to_string(),
                };
                octets[3] = 255;
                format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
            }
            Err(_) => "255.255.255.255".to_string(),
        }
    }
    fn set_up(&mut self) {
        let broadcast_ip = self.get_broadcast_address();
        let addr = format!("{}:{}", broadcast_ip, LISTEN_PORT);

        match UdpSocket::bind(&addr) {
            Ok(socket) => {
                socket.set_nonblocking(true).unwrap();
                socket.set_broadcast(true).unwrap();
                godot_print!("Listening on {}", addr);
                self.socket = Some(socket);
            }
            Err(e) => {
                godot_error!("Failed to bind {}: {}", addr, e);
            }
        }
    }
    #[func]
    fn joinby_ip(&mut self, ip: GString) {
        self.base_mut().emit_signal("join_game", &[ip.to_variant()]);
    }

    #[func]
    fn d(&mut self, ip: GString) {
        let mut peer = ENetMultiplayerPeer::new_gd();
        let error = peer.create_client(&ip.to_string(), 55555);
        if error == godot::global::Error::OK {
            self.base_mut()
                .get_multiplayer()
                .unwrap()
                .set_multiplayer_peer(&peer);
            self.base_mut()
                .get_tree()
                .change_scene_to_file("res://world/World.scn");
        } else {
            godot_error!("Failed to create client: {:?}", error);
        }
    }

    #[func]
    fn on_back_pressed(&mut self) {
        let mut scene_tree = self.base().get_tree();
        scene_tree.change_scene_to_file("res://SaveAndLoad/LoadMenu.scn");
    }
}
