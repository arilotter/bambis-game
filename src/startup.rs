use crate::{
    dude::{dude_hp_bar, DudeBundle},
    dungeon::{Dungeon, TileKind},
    prelude::*,
};

pub fn startup(mut commands: Commands) {
    // frame updating
    commands.insert_resource(LastFrame::default());
    commands.insert_resource(CurrentFrame::default());
    commands.insert_resource(CurrentSessionFrame::default());
    commands.insert_resource(ConfirmedFrame::default());
    commands.insert_resource(RollbackStatus::default());
    commands.insert_resource(ValidatableFrame::default());

    // desync detection
    commands.insert_resource(FrameHashes::default());
    commands.insert_resource(RxFrameHashes::default());

    // ggrs local players
    commands.insert_resource(LocalHandles::default());
    //commands.insert_resource(WrappedSessionType::default());

    // physics toggling
    commands.insert_resource(EnablePhysicsAfter::default());
    commands.insert_resource(PhysicsEnabled::default());

    // network timer
    commands.insert_resource(NetworkStatsTimer(Timer::from_seconds(
        2.0,
        TimerMode::Repeating,
    )))
}

pub fn reset_rapier(
    mut commands: Commands,
    mut rapier: ResMut<RapierContext>,
    collider_handles: Query<Entity, With<RapierColliderHandle>>,
    rb_handles: Query<Entity, With<RapierRigidBodyHandle>>,
) {
    // Force rapier to reload everything
    for e in collider_handles.iter() {
        commands.entity(e).remove::<RapierColliderHandle>();
    }
    for e in rb_handles.iter() {
        commands.entity(e).remove::<RapierRigidBodyHandle>();
    }

    // Re-initialize everything we overwrite with default values
    let context = RapierContext::default();
    rapier.bodies = context.bodies;
    rapier.colliders = context.colliders;
    rapier.broad_phase = context.broad_phase;
    rapier.narrow_phase = context.narrow_phase;
    rapier.ccd_solver = context.ccd_solver;
    rapier.impulse_joints = context.impulse_joints;
    rapier.integration_parameters = context.integration_parameters;
    rapier.islands = context.islands;
    rapier.multibody_joints = context.multibody_joints;
    rapier.pipeline = context.pipeline;
    rapier.query_pipeline = context.query_pipeline;

    // Add a bit more CCD
    rapier.integration_parameters.max_ccd_substeps = 5;

    if let Ok(context_bytes) = bincode::serialize(rapier.as_ref()) {
        let rapier_checksum = fletcher16(&context_bytes);
        log::trace!("Context hash at init: {}", rapier_checksum);

        commands.insert_resource(PhysicsRollbackState {
            rapier_state: Some(context_bytes),
            rapier_checksum,
        })
    } else {
        commands.insert_resource(PhysicsRollbackState::default());
    }
}

pub fn respawn_all(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    spawn_pool: Query<(Entity, &DeterministicSpawn)>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut dungeon = Dungeon::gen(2);
    commands.spawn(Camera2dBundle::default());

    // Everything must be spawned in the same order, every time,
    // deterministically.  There is also potential for bevy itself to return
    // queries to bevy_rapier that do not have the entities in the same order,
    // but in my experience with this example, that happens somewhat rarely.  A
    // patch to bevy_rapier is required to ensure some sort of Entity order
    // otherwise on the reading end, much like the below sorting of our spawn.
    // WARNING:  This is something on my branch only!  This is in bevy_rapier PR #233

    // Get our entities and sort them by the spawn component index
    let mut sorted_spawn_pool: Vec<(Entity, &DeterministicSpawn)> = spawn_pool.iter().collect();
    sorted_spawn_pool.sort_by_key(|e| e.1.index);
    // Get the Entities in reverse for easy popping
    let mut sorted_entity_pool: Vec<Entity> = sorted_spawn_pool.iter().map(|p| p.0).rev().collect();

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
    for (y, row) in dungeon.get_tiles().into_iter().enumerate() {
        for (x, tile) in row.into_iter().enumerate() {
            let mut e = commands.entity(sorted_entity_pool.pop().unwrap());
            e.remove::<DeterministicSpawn>();

            e.insert(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(match tile.kind {
                    TileKind::Empty => 0,
                    TileKind::Floor => 1,
                    TileKind::Wall => 2,
                }),
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(x as f32, y as f32, 0.0) * (TILE_SIZE as f32),
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
    for i in 0..=1 {
        let spawn = dungeon.get_spawn_point();
        let id = Rollback::new(rip.next_id());
        let dude = commands
            .entity(sorted_entity_pool.pop().unwrap())
            .insert(DudeBundle::new(
                i,
                id,
                asset_server.load("guy.png"),
                Vec2::new((spawn.0 * TILE_SIZE) as f32, (spawn.1 * TILE_SIZE) as f32),
                100,
            ))
            .remove::<DeterministicSpawn>()
            .id();
        commands.spawn(dude_hp_bar(dude));
    }
}
