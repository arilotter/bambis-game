use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_simple_stat_bars::StatBarsPlugin;
use bevy_slinet::client::{ClientConnections, ClientPlugin, PacketReceiveEvent};
use dude::{dude_hp_bar, DudeBundle};
use math::lerp;
use proto::{
    collision_groups::{COL_DUDE, COL_FILTER_DUDE, COL_TERRAIN},
    constants::{PLAYER_MOVE_SPEED, TILE_SIZE},
    dungeon::TileKind,
    ClientPacket, NetworkingConfig, ObjectInfo, PhysicsInfo, PhysicsObject, ServerPacket,
};

mod dude;
mod math;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_plugin(ClientPlugin::<NetworkingConfig>::connect("127.0.0.1:3000"))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(StatBarsPlugin)
        .insert_resource(ClearColor(Color::rgb_u8(255, 255, 255)))
        .add_system(update_system)
        .add_system(pin_camera_to_player_system)
        .add_system(packet_receive_system)
        .add_system(send_player_pos_system)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_system(
    connections: Res<ClientConnections<NetworkingConfig>>,
    mut controller: Query<(&mut KinematicCharacterController, &mut Transform)>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
    if let Ok(mut c) = controller.get_single_mut() {
        let mut velocity = Vec2::ZERO;
        for key in keys.get_pressed() {
            match key {
                KeyCode::W => velocity.y += PLAYER_MOVE_SPEED,
                KeyCode::S => velocity.y -= PLAYER_MOVE_SPEED,
                KeyCode::A => velocity.x -= PLAYER_MOVE_SPEED,
                KeyCode::D => velocity.x += PLAYER_MOVE_SPEED,
                _ => (),
            }
        }
        c.0.translation = Some(velocity);

        let window = windows.get_primary().unwrap();
        let v = Vec2::new(window.width() / 2.0, window.height() / 2.0);

        let cpos = window.cursor_position();
        if let Some(target) = cpos {
            let diff = target - v;
            let angle = diff.y.atan2(diff.x); // Add/sub FRAC_PI here optionally
            c.1.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle);
        }
        if buttons.pressed(MouseButton::Left) {
            let p_transform = c.1;
            let p_rot = p_transform.rotation.to_euler(EulerRot::ZXY).0;
            let proj_vel = Vec2::new(p_rot.cos(), p_rot.sin());
            for conn in connections.iter() {
                let _ = conn.send(ClientPacket::ShootBullet(PhysicsInfo {
                    ang_vel: 0.0,
                    pos: Vec2::new(p_transform.translation.x, p_transform.translation.y),
                    rot: p_transform.rotation.y,
                    vel: proj_vel * 3.0,
                }));
            }
        }
    }
}

fn pin_camera_to_player_system(
    player: Query<(&Transform, &KinematicCharacterController), With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    if player.is_empty() {
        return;
    }
    let (p_transform, p_movement) = player.single();
    let (cam_x, cam_y) = (camera.single().translation.x, camera.single().translation.y);
    camera.single_mut().translation.x = lerp(
        cam_x,
        p_transform.translation.x + p_movement.translation.map_or(0.0, |t| t.x),
        0.1,
    );
    camera.single_mut().translation.y = lerp(
        cam_y,
        p_transform.translation.y + p_movement.translation.map_or(0.0, |t| t.y),
        0.1,
    );
}

fn packet_receive_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<PacketReceiveEvent<NetworkingConfig>>,
    mut physics_objects: Query<(
        &mut Transform,
        &mut Velocity,
        &PhysicsObject,
        Option<&Player>,
    )>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for event in events.iter() {
        match &event.packet {
            ServerPacket::ObjectUpdate {
                info,
                ang_vel,
                id,
                pos,
                rot,
                vel,
            } => {
                let mut found = false;
                for mut c in physics_objects.iter_mut() {
                    if &c.2.id == id {
                        if c.3.is_none() {
                            c.0.translation.x = dbg!(pos.x);
                            c.0.translation.y = dbg!(pos.y);
                            c.0.rotation.y = *rot;

                            c.1.angvel = *ang_vel;
                            c.1.linvel = *vel;
                        }
                        found = true;
                        break;
                    }
                }
                if !found {
                    let mut spawned = commands.spawn(info.create_instance(*id));
                    match info {
                        ObjectInfo::Player { is_you } => {
                            spawned.insert(DudeBundle::new(
                                asset_server.load("guy.png"),
                                *pos,
                                100,
                            ));
                            if *is_you {
                                spawned.insert((
                                    Player,
                                    KinematicCharacterController {
                                        filter_groups: Some(CollisionGroups::new(
                                            COL_DUDE,
                                            COL_FILTER_DUDE,
                                        )),
                                        slide: true,
                                        ..default()
                                    },
                                ));
                            }
                        }
                        ObjectInfo::Bullet => {
                            spawned.insert(SpriteBundle {
                                texture: asset_server.load("bullet.png"),
                                ..default()
                            });
                        }
                        _ => {}
                    }

                    let spawned_id = spawned.id();

                    match info {
                        ObjectInfo::Player { .. } => {
                            commands.spawn(dude_hp_bar(spawned_id));
                        }
                        _ => {}
                    }
                }
            }
            ServerPacket::Dungeon { tiles } => {
                let texture_handle = asset_server.load("tiles.png");
                let texture_atlas = TextureAtlas::from_grid(
                    texture_handle,
                    Vec2 {
                        x: TILE_SIZE as f32,
                        y: TILE_SIZE as f32,
                    },
                    2,
                    2,
                    None,
                    None,
                );
                let texture_atlas_handle = texture_atlases.add(texture_atlas);
                // todo dedupe w server position code
                for (y, row) in tiles.into_iter().enumerate() {
                    for (x, tile) in row.into_iter().enumerate() {
                        let mut e = commands.spawn(SpriteSheetBundle {
                            sprite: TextureAtlasSprite::new(match tile.kind {
                                TileKind::Empty => 0,
                                TileKind::Floor => 1,
                                TileKind::Wall => 2,
                            }),
                            texture_atlas: texture_atlas_handle.clone(),
                            transform: Transform {
                                translation: Vec3::new(x as f32, y as f32, 0.0)
                                    * (TILE_SIZE as f32),
                                ..default()
                            },
                            ..default()
                        });
                        if matches!(tile.kind, TileKind::Wall) {
                            e.insert(RigidBody::Fixed)
                                .insert(Collider::cuboid(32.0, 32.0))
                                .insert(CollisionGroups::new(COL_TERRAIN, Group::ALL));
                        }
                    }
                }
            }
        }
    }
}

fn send_player_pos_system(
    connections: Res<ClientConnections<NetworkingConfig>>,
    player: Query<(&Transform, &Velocity), With<Player>>,
) {
    if let Ok(t) = player.get_single() {
        for conn in connections.iter() {
            conn.send(ClientPacket::PlayerMoved(PhysicsInfo {
                ang_vel: t.1.angvel,
                pos: Vec2::new(t.0.translation.x, t.0.translation.y),
                rot: t.0.rotation.y,
                vel: t.1.linvel,
            }))
            .unwrap();
        }
    }
}
