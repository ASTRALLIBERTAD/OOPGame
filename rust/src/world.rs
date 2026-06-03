use godot::classes::{INode2D, Node2D, PacketPeerUdp};
use godot::obj::NewGd;
use godot::prelude::*;
use local_ip_address::local_ip;

use crate::rustplayer::Rustplayer;

const BROADCAST_PORT: i32 = 8912;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct Node2dRust {
    #[base]
    base: Base<Node2D>,
    udp: Gd<PacketPeerUdp>,
    #[export]
    authority_player: OnEditor<Gd<Rustplayer>>,
    #[export]
    pub player_node_names: Array<GString>,
}

#[godot_api]
impl INode2D for Node2dRust {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            base,
            udp: PacketPeerUdp::new_gd(),
            authority_player: OnEditor::default(),
            player_node_names: Array::default(),
        }
    }
}

#[godot_api]
impl Node2dRust {
    #[func]
    fn broadcast(&mut self) {
        let result = self.udp.bind(0);
        if result != godot::global::Error::OK {
            godot_error!("Failed to bind UDP sender: {:?}", result);
            return;
        }
        self.udp.set_broadcast_enabled(true);
        godot_print!("UDP Broadcaster ready");
    }
    fn get_broadcast_address(&self) -> String {
        match local_ip() {
            Ok(ip) => {
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

    #[func]
    fn broadcaster_timeout(&mut self, packet: PackedByteArray) {
        let addr = GString::from(&self.get_broadcast_address().to_string());

        let result = self.udp.set_dest_address(&addr, BROADCAST_PORT);
        if result != godot::global::Error::OK {
            godot_error!("Failed to set dest address: {:?}", result);
            return;
        }

        let result = self.udp.put_packet(&packet);
        if result != godot::global::Error::OK {
            godot_error!("Failed to send packet: {:?}", result);
            return;
        }

        godot_print!("Broadcasted to {}:{}", addr, BROADCAST_PORT);
    }
}
