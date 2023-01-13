use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SPEED: f32 = 5.0;
const TILE_SIZE: usize = 16;

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
    let texture_handle = asset_server.load("tiles.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2 {
            x: TILE_SIZE as f32,
            y: TILE_SIZE as f32,
        },
        21,
        15,
        None,
        None,
    );
    let num_sprites = 21 + 15;
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for x in -40isize..40 {
        for y in -40isize..40 {
            commands.spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new((x + y).abs() as usize % num_sprites),
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(
                        (x * TILE_SIZE as isize) as f32,
                        (y * TILE_SIZE as isize) as f32,
                        0.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("guy.png"),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(16.0))
        .insert(KinematicCharacterController::default())
        .insert(Player);

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("guy.png"),
            transform: Transform {
                translation: Vec3::new(0.0, -64.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(16.0));

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("guy.png"),
            transform: Transform {
                translation: Vec3::new(64.0, 0.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(16.0));
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("guy.png"),
            transform: Transform {
                translation: Vec3::new(0.0, 64.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(16.0));
}

fn update_system(
    mut controllers: Query<(&mut KinematicCharacterController, &mut Transform)>,
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

        for (_, mut transform) in controllers.iter_mut() {
            transform.rotation = Quat::from_rotation_z(-angle - (PI / 4.0));
        }
    }
    for (mut controller, _) in controllers.iter_mut() {
        controller.translation = Some(velocity);
    }
}

fn pin_camera_to_player_system(
    player: Query<(&Transform, &KinematicCharacterController), With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let (p_transform, p_movement) = player.single();
    camera.single_mut().translation.x =
        p_transform.translation.x + p_movement.translation.map_or(0.0, |t| t.x);
    camera.single_mut().translation.y =
        p_transform.translation.y + p_movement.translation.map_or(0.0, |t| t.y);
}
