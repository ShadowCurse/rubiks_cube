use bevy::prelude::*;

mod camera;
mod cube_material;
mod cursor;
mod ray_extension;
mod rubiks_cube;
mod rubiks_cube_plugin;

use camera::{CameraControllerPlugin, OrbitCamera};
use cube_material::CubeMaterial;
use cursor::CursorRayPlugin;
use rubiks_cube_plugin::RubiksCubePlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    app.add_plugin(MaterialPlugin::<CubeMaterial>::default());

    app.add_plugin(CameraControllerPlugin);
    app.add_plugin(CursorRayPlugin);
    app.add_plugin(RubiksCubePlugin);

    app.add_startup_system(setup);

    app.run();
}

fn setup(mut commands: Commands) {
    // light
    commands.spawn(PointLightBundle {
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
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(OrbitCamera {
            radius: 1.0,
            ..Default::default()
        });
}
