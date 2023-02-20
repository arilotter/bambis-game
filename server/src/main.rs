use bevy::{prelude::*, time::FixedTimestep};
use bevy_rapier2d::prelude::*;
use bevy_slinet::{
    connection::ConnectionId,
    server::{NewConnectionEvent, PacketReceiveEvent, ServerConnections, ServerPlugin},
};
use dungeon::gen_dungeon;
use proto::{
    collision_groups::COL_TERRAIN,
    constants::TILE_SIZE,
    dungeon::{Tile, TileKind},
    *,
};

mod dungeon;
mod math;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(ServerPlugin::<NetworkingConfig>::bind("0.0.0.0:3000"))
        .add_system(new_connection_system)
        .add_system(packet_receive_system)
        .add_startup_system(gen_dungeon_system)
        .add_stage_after(
            CoreStage::Update,
            FixedUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.5))
                .with_system(send_physics_packets),
        )
        .run()
}

fn gen_dungeon_system(mut commands: Commands) {
    let dungeon = Dungeon::gen();
    for (y, row) in dungeon.tiles.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            if matches!(tile.kind, TileKind::Wall) {
                commands.spawn((
                    Transform {
                        translation: Vec3::new(x as f32, y as f32, 0.0) * (TILE_SIZE as f32),
                        ..default()
                    },
                    RigidBody::Fixed,
                    Collider::cuboid(32.0, 32.0),
                    CollisionGroups::new(COL_TERRAIN, Group::ALL),
                ));
            }
        }
    }
    commands.spawn(dungeon);
}

fn new_connection_system(
    mut commands: Commands,
    mut events: EventReader<NewConnectionEvent<NetworkingConfig>>,
    physics_objects: Query<&PhysicsObject>,
    mut dungeon: Query<&mut Dungeon>,
) {
    // associate a player ID with the connection
    for event in events.iter() {
        let mut d = dungeon.single_mut();
        event
            .connection
            .send(ServerPacket::Dungeon {
                tiles: d.get_tiles(),
            })
            .unwrap();
        let spawn_point = d.get_spawn_point();
        let mut new_ent = ObjectInfo::Player { is_you: false }
            .create_instance(get_next_phys_id(physics_objects.iter()));
        new_ent.1.transform.translation.x = (spawn_point.0 * TILE_SIZE) as f32;
        new_ent.1.transform.translation.y = (spawn_point.1 * TILE_SIZE) as f32;
        commands.spawn((new_ent, (event.connection.id())));
        println!("Spawned player with ID {:?}", event.connection.id());
    }
}

fn packet_receive_system(
    mut commands: Commands,
    mut events: EventReader<PacketReceiveEvent<NetworkingConfig>>,
    _connections: Res<ServerConnections<NetworkingConfig>>,
    mut physics_objects: Query<(
        &mut Transform,
        &mut Velocity,
        &PhysicsObject,
        Option<&ConnectionId>,
    )>,
) {
    for event in events.iter() {
        match &event.packet {
            ClientPacket::PlayerMoved(physics) => {
                let mut char = physics_objects
                    .iter_mut()
                    .find(|c| c.3 == Some(&event.connection.id()))
                    .unwrap();
                char.0.translation.x = physics.pos.x;
                char.0.translation.y = physics.pos.y;
                char.0.rotation.y = physics.rot;

                char.1.linvel = physics.vel;
                char.1.angvel = physics.ang_vel;
            }
            ClientPacket::ShootBullet(physics) => {
                eprintln!("shooting bullet {:?}", physics);
                let mut bullet = ObjectInfo::Bullet
                    .create_instance(get_next_phys_id(physics_objects.iter().map(|x| x.2)));
                bullet.1.transform.translation.x = physics.pos.x;
                bullet.1.transform.translation.y = physics.pos.y;
                bullet.1.transform.rotation.y = physics.rot;
                bullet.1.velocity.angvel = physics.ang_vel;
                bullet.1.velocity.linvel = physics.vel;
                commands.spawn(bullet);
            }
        }
    }
}

fn send_physics_packets(
    clients: Res<ServerConnections<NetworkingConfig>>,
    physics_objects: Query<(
        &PhysicsObject,
        &Transform,
        &Velocity,
        &ObjectInfo,
        Option<&ConnectionId>,
    )>,
) {
    println!("Num physics objects: {}", physics_objects.iter().count());
    for client in clients.iter() {
        for packet in physics_objects
            .iter()
            .map(
                |(physics, transform, vel, info, conn_id)| ServerPacket::ObjectUpdate {
                    info: match info {
                        ObjectInfo::Player { .. } if Some(&client.id()) == conn_id => {
                            ObjectInfo::Player { is_you: true }
                        }
                        _ => *info,
                    },
                    id: physics.id,
                    vel: vel.linvel,
                    ang_vel: vel.angvel,
                    pos: Vec2::new(transform.translation.x, transform.translation.y),
                    rot: transform.rotation.y,
                },
            )
        {
            let _ = client.send(packet);
        }
    }
}

#[derive(Component)]
struct Dungeon {
    tiles: Vec<Vec<Tile>>,
    spawn_points: Vec<(usize, usize)>,
    spawn_index: usize,
}

impl Dungeon {
    fn gen() -> Self {
        let tiles = gen_dungeon();
        let mut spawn_points = Vec::with_capacity(4);
        'outer: for (y, row) in tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if spawn_points.len() >= 4 {
                    break 'outer;
                }
                if matches!(tile.kind, TileKind::Floor) {
                    spawn_points.push((x, y))
                }
            }
        }
        Self {
            tiles,
            spawn_points,
            spawn_index: 0,
        }
    }
    fn get_spawn_point(&mut self) -> (usize, usize) {
        let point = self.spawn_points[self.spawn_index];
        self.spawn_index += 1;
        self.spawn_index %= self.spawn_points.len();
        point
    }
    fn get_tiles(&self) -> Vec<Vec<Tile>> {
        self.tiles.clone()
    }
}
