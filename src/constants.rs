use bevy_rapier2d::geometry::Group;

pub const COL_DUDE: Group = Group::GROUP_1;
pub const COL_BULLET: Group = Group::GROUP_2;
pub const COL_TERRAIN: Group = Group::GROUP_3;

pub const COL_FILTER_BULLET: Group =
    Group::from_bits_truncate(COL_TERRAIN.bits() | COL_BULLET.bits());
pub const COL_FILTER_DUDE: Group = COL_TERRAIN;

pub const TILE_SIZE: usize = 64;
pub const PLAYER_MOVE_SPEED: f32 = 5.0;
