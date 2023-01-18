use std::{f32::consts::PI, time::Instant};

use bevy::{prelude::*, reflect::erased_serde::__private::serde::__private::de};
use bevy_rapier2d::prelude::*;
use constants::{SPEED, TILE_SIZE};
use dungeon::{gen_dungeon, Room};
use math::{iter_float, lerp};
use rand::Rng;

use crate::dungeon::TileKind;

mod constants;
mod dungeon;
mod math;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup)
        .add_system(update_system)
        .add_system(pin_camera_to_player_system)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());

    let now = Instant::now();

    let dungeon = gen_dungeon();
    println!("{}", now.elapsed().as_millis());

    // for row in dungeon.1 {
    //     for tile in row {
    //         eprint!(
    //             "{}",
    //             match tile.kind {
    //                 TileKind::Empty => "  ",
    //                 TileKind::Floor => "░░",
    //                 TileKind::Wall => "██",
    //             }
    //         );
    //     }
    //     eprintln!("");
    // }
    // for room in rooms {
    //     commands
    //         .spawn(SpriteBundle {
    //             sprite: Sprite {
    //                 color: Color::rgb(
    //                     rng.gen_range(0.0..1.0),
    //                     rng.gen_range(0.0..1.0),
    //                     rng.gen_range(0.0..1.0),
    //                 ),
    //                 custom_size: Some(Vec2::new(room.2, room.3)),
    //                 ..Default::default()
    //             },
    //             ..Default::default()
    //         })
    //         .insert(Transform {
    //             translation: Vec3::new(room.0, room.1, 0.0),
    //             ..Default::default()
    //         });
    // }
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

    let mut spawn_point = Vec2::new(0.0, 0.0);
    for (y, row) in dungeon.1.into_iter().enumerate() {
        for (x, tile) in row.into_iter().enumerate() {
            let mut e = commands.spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(match tile.kind {
                    TileKind::Empty => 0,
                    TileKind::Floor => 1,
                    TileKind::Wall => 2,
                }),
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(x as f32, y as f32, 0.0) * (TILE_SIZE as f32),
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

fn update_system(
    mut commands: Commands,
    mut controller: Query<(&mut KinematicCharacterController, &mut Transform)>,
    asset_server: Res<AssetServer>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
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

    let window = windows.get_primary().unwrap();
    let v = Vec2::new(window.width() / 2.0, window.height() / 2.0);

    let cpos = window.cursor_position();
    if let Some(target) = cpos {
        let angle = (target - v).angle_between(v);

        controller.single_mut().1.rotation = Quat::from_rotation_z(-angle - (PI / 4.0));
    }
    controller.single_mut().0.translation = Some(velocity);
    if buttons.pressed(MouseButton::Left) {
        let p_transform = controller.single().1;
        let p_x_y_dir = p_transform.rotation.to_euler(EulerRot::ZXY).0;
        println!("{}", p_x_y_dir);
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("bullet.png"),
                transform: Transform {
                    translation: p_transform.translation,
                    rotation: p_transform.rotation,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Sensor)
            .insert(Collider::cuboid(16.0, 1.0))
            .insert(LockedAxes::TRANSLATION_LOCKED)
            .insert(Velocity {
                linvel: Vec2::new(1.0, 2.0),
                angvel: 0.0,
            });
    }
}

fn pin_camera_to_player_system(
    player: Query<(&Transform, &KinematicCharacterController), With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
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
