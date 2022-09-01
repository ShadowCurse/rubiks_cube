use crate::ray::Ray;
use bevy::prelude::*;

pub struct CursorRayPlugin;

impl Plugin for CursorRayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorRay::default());
        app.add_system(world_cursor_system);
        // app.add_system(debug_ray);
    }
}

#[derive(Resource, Debug, Default)]
pub struct CursorRay(pub Ray);

fn world_cursor_system(
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut crs: ResMut<CursorRay>,
) {
    if let Ok((camera, camera_transform)) = camera.get_single() {
        let window = windows.get_primary().unwrap();
        if let Some(screen_pos) = window.cursor_position() {
            if let Some(ray) = Ray::from_screenspace(screen_pos, camera, camera_transform) {
                crs.0 = ray;
            }
        }
    }
}

// fn debug_ray(
//     crs: Res<CursorRay>,
//     key_input: Res<Input<KeyCode>>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     if key_input.pressed(KeyCode::Space) {
//         commands.spawn_bundle(PbrBundle {
//             mesh: meshes.add(
//                 shape::UVSphere {
//                     radius: 0.01,
//                     ..default()
//                 }
//                 .into(),
//             ),
//             material: materials.add(Color::RED.into()),
//             transform: Transform::from_translation(crs.origin.into()),
//             ..default()
//         });
//         for i in 1..50 {
//             commands.spawn_bundle(PbrBundle {
//                 mesh: meshes.add(
//                     shape::UVSphere {
//                         radius: 0.005,
//                         ..default()
//                     }
//                     .into(),
//                 ),
//                 material: materials.add(Color::GREEN.into()),
//                 transform: Transform::from_translation(
//                     (crs.origin + crs.direction * i as f32 / 30.0).into(),
//                 ),
//                 ..default()
//             });
//         }
//     }
// }
