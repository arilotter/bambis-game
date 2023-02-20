use crate::prelude::*;

#[derive(Bundle)]
pub struct BulletBundle {
    sprite: SpriteBundle,
    velocity: Velocity,
    rigid_body: RigidBody,
    collider: Collider,
    collision_groups: CollisionGroups,
    rollback: Rollback,
    name: Name,
}

impl BulletBundle {
    pub fn new(player_transform: &Transform, rollback: Rollback, texture: Handle<Image>) -> Self {
        let p_rot = player_transform.rotation.to_euler(EulerRot::ZXY).0;

        Self {
            name: Name::new(format!("Bullet")),
            rollback,
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: Vec3::new(
                        player_transform.translation.x,
                        player_transform.translation.y,
                        1.0,
                    ),
                    rotation: player_transform.rotation.clone(),
                    ..default()
                },
                ..default()
            },
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(16.0, 1.0),
            collision_groups: CollisionGroups::new(COL_BULLET, COL_FILTER_BULLET),
            velocity: Velocity {
                linvel: Vec2::new(p_rot.cos(), p_rot.sin()) * 1000.0,
                angvel: 0.0,
            },
        }
    }
}
