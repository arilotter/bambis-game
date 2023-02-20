use crate::prelude::*;
use bevy_simple_stat_bars::{observers::StatBarObserver, prelude::*};

#[derive(Bundle)]
pub struct DudeBundle {
    sprite: SpriteBundle,
    rigid_body: RigidBody,
    collider: Collider,
    health: Health,
    collision_groups: CollisionGroups,
    player: Player,
    rollback: Rollback,
    name: Name,
    controller: KinematicCharacterController,
}

impl DudeBundle {
    pub fn new(
        player: usize,
        rollback: Rollback,
        texture: Handle<Image>,
        spawn_point: Vec2,
        max_hp: usize,
    ) -> Self {
        Self {
            name: Name::new(format!("Player {}", player)),
            rollback,
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: Vec3::new(spawn_point.x, spawn_point.y, 1.0),
                    ..default()
                },
                ..default()
            },
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::ball(16.0),
            collision_groups: CollisionGroups::new(COL_DUDE, COL_FILTER_DUDE),
            health: Health::new(max_hp),
            player: Player { handle: player },
            controller: KinematicCharacterController {
                filter_groups: Some(CollisionGroups::new(COL_DUDE, COL_FILTER_DUDE)),
                slide: true,
                ..default()
            },
        }
    }
}

pub fn dude_hp_bar(
    entity: Entity,
) -> (
    StatBarColor,
    StatBarEmptyColor,
    StatBarBorder,
    StatBarValue,
    StatBarSize,
    StatBarSubject,
    StatBarPosition,
    StatBarZDepth,
    StatBarObserver,
) {
    (
        StatBarColor(Color::GREEN),
        StatBarEmptyColor(Color::BLACK),
        StatBarBorder {
            color: Color::DARK_GRAY,
            thickness: 3.0,
        },
        StatBarValue(1.0),
        StatBarSize {
            full_length: 50.0,
            thickness: 6.0,
        },
        StatBarSubject(entity),
        StatBarPosition(40.0 * Vec2::Y),
        StatBarZDepth(2.0),
        component_observer(|h: &Health| h.hp as f32 / h.max as f32),
    )
}
