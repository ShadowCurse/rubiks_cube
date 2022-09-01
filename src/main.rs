use bevy::prelude::*;
// use bevy_editor_pls::prelude::*;

mod camera;
mod cursor;
mod ray;
mod rubiks_cube;

use camera::{CameraController, CameraControllerPlugin};
use cursor::CursorRayPlugin;
use rubiks_cube::RubiksCubePlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    // app.add_plugin(EditorPlugin);

    app.add_plugin(CameraControllerPlugin);
    app.add_plugin(CursorRayPlugin);
    app.add_plugin(RubiksCubePlugin);

    app.add_startup_system(setup);

    app.run();
}

fn setup(mut commands: Commands) {
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(CameraController::default());
}
