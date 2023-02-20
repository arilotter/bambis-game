use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use collision_groups::*;
use serde::{Deserialize, Serialize};

use bevy_slinet::packet_length_serializer::LittleEndian;
use bevy_slinet::protocols::udp::UdpProtocol;
use bevy_slinet::serializers::bincode::{BincodeSerializer, DefaultOptions};
use bevy_slinet::{ClientConfig, ServerConfig};

pub mod collision_groups;
pub mod constants;
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PhysicsInfo {
    pub pos: Vec2,
    pub rot: f32,
    pub vel: Vec2,
    pub ang_vel: f32,
}

// todo include timestamps for rollback
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientPacket {
    PlayerMoved(PhysicsInfo),
    ShootBullet(PhysicsInfo),
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ObjectInfo {
    Player { is_you: bool },
    Bingus,
    Bullet,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerPacket {
    ObjectUpdate {
        id: u64,
        pos: Vec2,
        rot: f32,
        vel: Vec2,
        ang_vel: f32,
        info: ObjectInfo,
    },
    Dungeon {
        tiles: Vec<Vec<dungeon::Tile>>,
    },
}

#[derive(Component, Debug)]
pub struct PhysicsObject {
    pub id: u64,
}

impl PhysicsObject {
    pub fn new(id: u64) -> Self {
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

#[derive(Bundle)]
pub struct PhysicsObjectBundle {
    pub physics_object: PhysicsObject,
    pub transform: Transform,
    global_transform: GlobalTransform,
    pub velocity: Velocity,
    pub rigid_body: RigidBody,
    gravity_scale: GravityScale,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    ccd: Ccd,
}

pub fn new_physics_object<'a>(id: u64) -> PhysicsObjectBundle {
    PhysicsObjectBundle {
        physics_object: PhysicsObject { id },
        gravity_scale: GravityScale(0.0),
        ccd: Ccd::enabled(),
        transform: default(),
        global_transform: default(),
        collider: default(),
        collision_groups: default(),
        rigid_body: default(),
        velocity: default(),
    }
}

pub fn get_next_phys_id<'a>(query: impl Iterator<Item = &'a PhysicsObject>) -> u64 {
    query.map(|c| c.id).max().unwrap_or(0) + 1
}

impl ObjectInfo {
    pub fn create_instance<'a>(self, id: u64) -> (ObjectInfo, PhysicsObjectBundle) {
        let mut new_ent = new_physics_object(id);

        match self {
            Self::Player { .. } => {
                new_ent.rigid_body = RigidBody::KinematicPositionBased;
                new_ent.collider = Collider::ball(16.0);
                new_ent.collision_groups = CollisionGroups::new(COL_DUDE, COL_FILTER_DUDE);
            }
            Self::Bullet => {
                new_ent.collider = Collider::cuboid(16.0, 1.0);
                new_ent.collision_groups = CollisionGroups::new(COL_BULLET, COL_FILTER_BULLET);
            }
            Self::Bingus => {}
        }
        (self, new_ent)
    }
}
