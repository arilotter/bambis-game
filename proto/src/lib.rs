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
pub enum CharacterType {
    Player,
    Bingus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerPacket {
    CharacterPosition(usize, Vec2, CharacterType),
    IdentifyClient(usize),
    Dungeon {
        tiles: Vec<Vec<dungeon::Tile>>,
        spawn_point: (usize, usize),
    },
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

#[derive(Component)]
pub struct Health {
    pub hp: usize,
    pub max: usize,
}

impl Health {
    pub fn new(max: usize) -> Self {
        Self { max, hp: max }
    }
}
