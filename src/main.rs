#![feature(is_sorted)]
use bevy::prelude::*;

mod audio;
mod camera;
mod cube_material;
mod cursor;
mod game_state;
mod ray_extension;
mod rubiks_cube;
mod rubiks_cube_plugin;
mod ui;

use audio::AudioPlugin;
use camera::{CameraControllerPlugin, OrbitCamera};
use cube_material::CubeMaterial;
use cursor::CursorRayPlugin;
use game_state::GameStatePlugin;
use rubiks_cube_plugin::RubiksCubePlugin;
use ui::UiPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStates {
    MainMenu,
    InGame,
    Paused,
    EndGame,
}

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::BLACK));
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.4,
    });

    app.add_plugins(DefaultPlugins);
    app.add_plugin(MaterialPlugin::<CubeMaterial>::default());

    app.add_state(GameStates::MainMenu);

    app.add_plugin(AudioPlugin);
    app.add_plugin(UiPlugin);
    app.add_plugin(CameraControllerPlugin);
    app.add_plugin(GameStatePlugin);
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
        transform: Transform::from_xyz(2.0, 4.0, 2.0),
        ..default()
    });
    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.5, 0.2, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(OrbitCamera {
            radius: 1.0,
            ..Default::default()
        });
}
