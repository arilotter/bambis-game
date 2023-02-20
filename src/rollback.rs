use ggrs::Config;

use crate::{bullet::BulletBundle, prelude::*};

#[rustfmt::skip]
mod input_bits {
    pub const ANGLE_RANGE: f32 = 2.0 * std::f32::consts::PI;
    pub const ANGLE_OFFSET: f32 = std::f32::consts::PI;

    pub const UP:        u16 = 0b0000010000000000;
    pub const DOWN:      u16 = 0b0000100000000000;
    pub const LEFT:      u16 = 0b0001000000000000;
    pub const RIGHT:     u16 = 0b0010000000000000;
    pub const PRIMARY:   u16 = 0b0100000000000000;
    pub const SECONDARY: u16 = 0b1000000000000000;
    pub const ANGLE:     u16 = 0b0000001111111111;
}
/// GGRS player handle, we use this to associate GGRS handles back to our [`Entity`]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Component)]
pub struct Player {
    pub handle: usize,
}

/// Local handles, this should just be 1 entry in this demo, but you may end up wanting to implement 2v2
#[derive(Default, Resource)]
pub struct LocalHandles {
    pub handles: Vec<PlayerHandle>,
}

/// The main GGRS configuration type
#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = GGRSInput;
    // bevy_ggrs doesn't really use State, so just make this a small whatever
    type State = u8;
    type Address = String;
}

#[derive(Clone, Copy, Debug)]
struct PlayerInput {
    pub angle: f32,
    pub primary: bool,
    pub secondary: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl PlayerInput {
    pub fn movement_vec(&self) -> Vec2 {
        Vec2::new(
            (self.right as i8 - self.left as i8) as f32 * PLAYER_MOVE_SPEED,
            (self.up as i8 - self.down as i8) as f32 * PLAYER_MOVE_SPEED,
        )
    }
}

impl From<u16> for PlayerInput {
    fn from(input: u16) -> Self {
        PlayerInput {
            angle: (input & input_bits::ANGLE) as f32 / 1024.0 * input_bits::ANGLE_RANGE
                - input_bits::ANGLE_OFFSET,
            primary: input & input_bits::PRIMARY != 0,
            secondary: input & input_bits::SECONDARY != 0,
            up: input & input_bits::UP != 0,
            down: input & input_bits::DOWN != 0,
            left: input & input_bits::LEFT != 0,
            right: input & input_bits::RIGHT != 0,
        }
    }
}

impl From<PlayerInput> for u16 {
    fn from(input: PlayerInput) -> Self {
        let angle =
            ((input.angle + input_bits::ANGLE_OFFSET) / input_bits::ANGLE_RANGE * 1024.0) as u16;

        let primary = if input.primary {
            input_bits::PRIMARY
        } else {
            0
        };

        let secondary = if input.secondary {
            input_bits::SECONDARY
        } else {
            0
        };

        let up = if input.up { input_bits::UP } else { 0 };
        let down = if input.down { input_bits::DOWN } else { 0 };
        let left = if input.left { input_bits::LEFT } else { 0 };
        let right = if input.right { input_bits::RIGHT } else { 0 };
        angle | primary | secondary | up | down | left | right
    }
}

/// Our primary data struct; what players send to one another
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
pub struct GGRSInput {
    // The input from our player
    pub input: u16,

    // Desync detection
    pub last_confirmed_hash: u16,
    pub last_confirmed_frame: Frame,
    // Ok, so I know what you're thinking:
    // > "That's not input!"
    // Well, you're right, and we're going to abuse the existing socket to also
    // communicate about the last confirmed frame we saw and what was the hash
    // of the physics state.  This allows us to detect desync.  This could also
    // use a new socket, but who wants to hole punch twice?  I have been working
    // on a GGRS branch (linked below) that introduces a new message type, but
    // it is not ready.  However, input-packing works good enough for now.
    // https://github.com/cscorley/ggrs/tree/arbitrary-messages-0.8
}

pub fn input(
    handle: In<PlayerHandle>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    physics_enabled: Res<PhysicsEnabled>,
    mut hashes: ResMut<FrameHashes>,
    validatable_frame: Res<ValidatableFrame>,
    windows: Res<Windows>,
) -> GGRSInput {
    let mut last_confirmed_frame = ggrs::NULL_FRAME;
    let mut last_confirmed_hash = 0;

    // Find a hash that we haven't sent yet.
    // This probably seems like overkill but we have to track a bunch anyway, we
    // might as well do our due diligence and inform our opponent of every hash
    // we have This may mean we ship them out of order.  The important thing is
    // we determine the desync *eventually* because that match is pretty much
    // invalidated without a state synchronization mechanism (which GGRS/GGPO
    // does not have out of the box.)
    for frame_hash in hashes.0.iter_mut() {
        // only send confirmed frames that have not yet been sent that are well past our max prediction window
        if frame_hash.confirmed
            && !frame_hash.sent
            && validatable_frame.is_validatable(frame_hash.frame)
        {
            log::trace!("Sending data {:?}", frame_hash);
            last_confirmed_frame = frame_hash.frame;
            last_confirmed_hash = frame_hash.rapier_checksum;
            frame_hash.sent = true;
        }
    }

    // Do not do anything until physics are live
    if !physics_enabled.0 {
        return GGRSInput {
            input: 0,
            last_confirmed_frame,
            last_confirmed_hash,
        };
    }

    let window = windows.get_primary().unwrap();
    let v = Vec2::new(window.width() / 2.0, window.height() / 2.0);

    let input = PlayerInput {
        angle: window
            .cursor_position()
            .map(|target| {
                let diff = target - v;
                diff.y.atan2(diff.x)
            })
            .unwrap_or(0.0),
        primary: mouse_input.just_pressed(MouseButton::Left),
        secondary: mouse_input.just_pressed(MouseButton::Right),
        up: keyboard_input.pressed(KeyCode::W),
        down: keyboard_input.pressed(KeyCode::S),
        left: keyboard_input.pressed(KeyCode::A),
        right: keyboard_input.pressed(KeyCode::D),
    };

    GGRSInput {
        input: input.into(),
        last_confirmed_frame,
        last_confirmed_hash,
    }
}

pub fn apply_inputs(
    mut commands: Commands,
    mut query: Query<(&mut KinematicCharacterController, &mut Transform, &Player)>,
    inputs: Res<PlayerInputs<GGRSConfig>>,
    mut hashes: ResMut<RxFrameHashes>,
    local_handles: Res<LocalHandles>,
    physics_enabled: Res<PhysicsEnabled>,
    mut rip: ResMut<RollbackIdProvider>,
    asset_server: Res<AssetServer>,
    spawn_pool: Query<(Entity, &DeterministicSpawn)>,
) {
    // Get our entities and sort them by the spawn component index
    let mut sorted_spawn_pool: Vec<(Entity, &DeterministicSpawn)> = spawn_pool.iter().collect();
    sorted_spawn_pool.sort_by_key(|e| e.1.index);
    // Get the Entities in reverse for easy popping
    let mut sorted_entity_pool: Vec<Entity> = sorted_spawn_pool.iter().map(|p| p.0).rev().collect();

    for (mut controller, mut transform, player) in query.iter_mut() {
        let (game_input, input_status) = inputs[player.handle];
        // Check the desync for this player if they're not a local handle
        // Did they send us some goodies?
        if !local_handles.handles.contains(&player.handle) && game_input.last_confirmed_frame > 0 {
            log::trace!("Got frame data {:?}", game_input);
            if let Some(frame_hash) = hashes
                .0
                .get_mut((game_input.last_confirmed_frame as usize) % DESYNC_MAX_FRAMES)
            {
                assert!(
                    frame_hash.frame != game_input.last_confirmed_frame
                        || frame_hash.rapier_checksum == game_input.last_confirmed_hash,
                    "Got new data for existing frame data {}",
                    frame_hash.frame
                );

                // Only update this local data if the frame is new-to-us.
                // We don't want to overwrite any existing validated status
                // unless the frame is replacing what is already in the buffer.
                if frame_hash.frame != game_input.last_confirmed_frame {
                    frame_hash.frame = game_input.last_confirmed_frame;
                    frame_hash.rapier_checksum = game_input.last_confirmed_hash;
                    frame_hash.validated = false;
                }
            }
        }

        // On to the boring stuff
        let input: PlayerInput = match input_status {
            InputStatus::Confirmed => game_input.input,
            InputStatus::Predicted => game_input.input,
            InputStatus::Disconnected => 0, // disconnected players do nothing
        }
        .into();

        if u16::from(input) > 0 {
            // Useful for desync observing
            log::trace!(
                "input {:?} from {}: {:?}",
                input_status,
                player.handle,
                input
            );
        }

        // Do not do anything until physics are live
        // This is a poor mans emulation to stop accidentally tripping velocity updates
        if !physics_enabled.0 {
            continue;
        }

        controller.translation = Some(input.movement_vec());

        transform.rotation = Quat::from_rotation_z(input.angle);

        if input.primary {
            let id = Rollback::new(rip.next_id());

            let bullet_bundle = BulletBundle::new(&*transform, id, asset_server.load("bullet.png"));
            commands
                .entity(sorted_entity_pool.pop().unwrap())
                .insert(bullet_bundle)
                .remove::<DeterministicSpawn>();
        }
    }
}

pub fn force_update_rollbackables(
    mut t_query: Query<&mut Transform, With<Rollback>>,
    mut v_query: Query<&mut Velocity, With<Rollback>>,
) {
    for mut t in t_query.iter_mut() {
        t.set_changed();
    }
    for mut v in v_query.iter_mut() {
        v.set_changed();
    }
}

#[test]
fn test_input_encode_decode() {
    for angle in iter_float(0.0..=input_bits::ANGLE_RANGE, 0.1) {
        for left in [true, false] {
            for right in [true, false] {
                for up in [true, false] {
                    for down in [true, false] {
                        for primary in [true, false] {
                            for secondary in [true, false] {
                                let input = PlayerInput {
                                    angle,
                                    left,
                                    right,
                                    up,
                                    down,
                                    primary,
                                    secondary,
                                };
                                assert_eq!(
                                    u16::from(input),
                                    u16::from(PlayerInput::from(u16::from(input)))
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
