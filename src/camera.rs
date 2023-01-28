use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

use crate::GameStates;

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OrbitCameraKeys::default());

        app.add_system_set(SystemSet::on_update(GameStates::InGame).with_system(pan_orbit_camera));
    }
}

#[derive(Resource)]
pub struct OrbitCameraKeys {
    orbit_button: MouseButton,
}

impl Default for OrbitCameraKeys {
    fn default() -> Self {
        Self {
            orbit_button: MouseButton::Right,
        }
    }
}

#[derive(Component)]
pub struct OrbitCamera {
    pub focus_point: Vec3,
    pub radius: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            focus_point: Vec3::ZERO,
            radius: 5.0,
        }
    }
}

fn pan_orbit_camera(
    windows: Res<Windows>,
    input_mouse: Res<Input<MouseButton>>,
    orbit_camera_keys: Res<OrbitCameraKeys>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;

    if input_mouse.pressed(orbit_camera_keys.orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    if let (Ok((mut pan_orbit, mut transform)), Some(window)) =
        (query.get_single_mut(), windows.get_primary())
    {
        if rotation_move.length_squared() > 0.0 {
            let delta_x = rotation_move.x / window.width() * std::f32::consts::PI * 2.0;
            let delta_y = rotation_move.y / window.height() * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation * pitch;
        } else if scroll.abs() > 0.0 {
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        let rot_matrix = Mat3::from_quat(transform.rotation);
        transform.translation =
            pan_orbit.focus_point + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
    }
}
