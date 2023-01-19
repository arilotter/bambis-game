use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_slinet::client::{ClientConnections, ClientPlugin, PacketReceiveEvent};
use constants::{SPEED, TILE_SIZE};
use math::lerp;
use proto::{dungeon::TileKind, Character, ClientPacket, NetworkingConfig, ServerPacket};

mod constants;
mod math;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_plugin(ClientPlugin::<NetworkingConfig>::connect(
            "192.168.2.127:3000",
        ))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
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
    mut commands: Commands,
    mut controller: Query<(&mut KinematicCharacterController, &mut Transform)>,
    asset_server: Res<AssetServer>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
    if let Ok(mut c) = controller.get_single_mut() {
        let mut velocity = Vec2::ZERO;
        for key in keys.get_pressed() {
            match key {
                KeyCode::W => velocity.y += SPEED,
                KeyCode::S => velocity.y -= SPEED,
                KeyCode::A => velocity.x -= SPEED,
                KeyCode::D => velocity.x += SPEED,
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
            commands
                .spawn(SpriteBundle {
                    texture: asset_server.load("bullet.png"),
                    transform: Transform {
                        translation: p_transform.translation
                            + Vec3::new(proj_vel.x, proj_vel.y, 0.0),
                        rotation: p_transform.rotation,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(RigidBody::KinematicVelocityBased)
                .insert(Collider::cuboid(16.0, 1.0))
                .insert(Sensor)
                .insert(Velocity {
                    linvel: proj_vel * 1000.0,
                    angvel: 0.0,
                });
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
    mut characters: Query<(&mut Transform, &Character)>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    // mut player: Query<Option<&Character>, With<Player>>,
) {
    for event in events.iter() {
        match &event.packet {
            ServerPacket::PlayerPosition(player, pos) => {
                let mut found = false;
                for mut c in characters.iter_mut() {
                    if &c.1.id == player {
                        eprintln!("x {} y {}", pos.x, pos.y);
                        c.0.translation.x = pos.x;
                        c.0.translation.y = pos.y;
                        found = true;
                        break;
                    }
                }
                if !found {
                    eprintln!("Spawning!");
                    commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("guy.png"),
                            transform: Transform {
                                translation: Vec3::new(pos.x, pos.y, 1.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(RigidBody::KinematicPositionBased)
                        .insert(Collider::ball(16.0))
                        .insert(Character::new(*player));
                }
            }
            ServerPacket::IdentifyClient(id) => {}
            ServerPacket::Dungeon(dungeon) => {
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
                eprintln!("spwaning dungeon");
                let mut spawn_point = Vec2::new(0.0, 0.0);
                for (y, row) in dungeon.into_iter().enumerate() {
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
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                        if matches!(tile.kind, TileKind::Wall) {
                            e.insert(RigidBody::Fixed)
                                .insert(Collider::cuboid(32.0, 32.0));
                        } else if matches!(tile.kind, TileKind::Floor) {
                            spawn_point = Vec2::new(x as f32, y as f32) * (TILE_SIZE as f32);
                        }
                    }
                }
                commands
                    .spawn(SpriteBundle {
                        texture: asset_server.load("guy.png"),
                        transform: Transform {
                            translation: Vec3::new(spawn_point.x, spawn_point.y, 1.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(RigidBody::KinematicPositionBased)
                    .insert(Collider::ball(16.0))
                    .insert(KinematicCharacterController::default())
                    .insert(Player);
            }
        }
    }
}

fn send_player_pos_system(
    connections: Res<ClientConnections<NetworkingConfig>>,
    player: Query<&Transform, With<Player>>,
) {
    if let Ok(t) = player.get_single() {
        for conn in connections.iter() {
            conn.send(ClientPacket::Position(Vec2::new(
                t.translation.x,
                t.translation.y,
            )))
            .unwrap();
        }
    }
}
