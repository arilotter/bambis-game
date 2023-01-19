use bevy::prelude::{Component, Vec2};
use serde::{Deserialize, Serialize};

use bevy_slinet::packet_length_serializer::LittleEndian;
use bevy_slinet::protocols::udp::UdpProtocol;
use bevy_slinet::serializers::bincode::{BincodeSerializer, DefaultOptions};
use bevy_slinet::{ClientConfig, ServerConfig};

pub mod dungeon;

pub struct NetworkingConfig;

impl ClientConfig for NetworkingConfig {
    type ClientPacket = ClientPacket;
    type ServerPacket = ServerPacket;
    type Protocol = UdpProtocol;
    type Serializer = BincodeSerializer<DefaultOptions>;
    type LengthSerializer = LittleEndian<u32>;
}

impl ServerConfig for NetworkingConfig {
    type ClientPacket = ClientPacket;
    type ServerPacket = ServerPacket;
    type Protocol = UdpProtocol;
    type Serializer = BincodeSerializer<DefaultOptions>;
    type LengthSerializer = LittleEndian<u32>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientPacket {
    Position(Vec2),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerPacket {
    PlayerPosition(usize, Vec2),
    IdentifyClient(usize),
    Dungeon(Vec<Vec<dungeon::Tile>>),
}

#[derive(Component)]
pub struct Character {
    pub id: usize,
}
impl Character {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}
