use crate::prelude::*;
pub fn pin_camera_to_player_system(
    player: Query<(&Transform, &KinematicCharacterController, &Player)>,
    local_handles: Res<LocalHandles>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    for (p_transform, p_movement, p_handle) in &player {
        if !local_handles.handles.contains(&p_handle.handle) {
            continue;
        }
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
}
