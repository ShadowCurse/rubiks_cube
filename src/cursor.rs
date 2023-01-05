use crate::ray::Ray;
use bevy::prelude::*;

pub struct CursorRayPlugin;

impl Plugin for CursorRayPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init);
        app.add_system(world_cursor_system);
        app.add_system(cursor_selection_vector);
        app.add_system(selection_vector_colliniar_axis);

        // app.add_startup_system(debug_axis);
        // app.add_system(debug_ray);
    }
}

#[derive(Resource, Debug, Default)]
pub struct CursorRay(pub Ray);

#[derive(Debug, Default)]
pub struct ScreenVector {
    start: Vec2,
    end: Vec2,
}

impl ScreenVector {
    fn vec(&self) -> Vec2 {
        self.end - self.start
    }
}

#[derive(Resource, Debug, Default)]
pub struct CursorSelectionVector(pub Option<ScreenVector>);

#[derive(Resource, Debug, Default)]
pub struct CursorCollinearAxis(pub Option<Vec3>);

fn init(mut commands: Commands) {
    commands.insert_resource(CursorRay::default());
    commands.insert_resource(CursorSelectionVector::default());
    commands.insert_resource(CursorCollinearAxis::default());
}

fn world_cursor_system(
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut crs: ResMut<CursorRay>,
) {
    if let Ok((camera, camera_transform)) = camera.get_single() {
        let window = windows
            .get_primary()
            .expect("We need a primary window to play the game");
        if let Some(screen_pos) = window.cursor_position() {
            if let Some(ray) = Ray::from_screenspace(screen_pos, camera, camera_transform) {
                crs.0 = ray;
            }
        }
    }
}

fn cursor_selection_vector(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut crs_vector: ResMut<CursorSelectionVector>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let window = windows
            .get_primary()
            .expect("We need a primary window to play the game");
        if let Some(screen_pos) = window.cursor_position() {
            crs_vector.0 = Some(ScreenVector {
                start: screen_pos,
                end: screen_pos,
            });
        }
    } else if mouse_input.pressed(MouseButton::Left) {
        let window = windows
            .get_primary()
            .expect("We need a primary window to play the game");
        if let Some(screen_pos) = window.cursor_position() {
            if let Some(ref mut vec) = crs_vector.0 {
                vec.end = screen_pos;
            }
        }
    } else if mouse_input.just_released(MouseButton::Left) {
        crs_vector.0 = None;
    } else {
    }
}

fn selection_vector_colliniar_axis(
    crs_vector: Res<CursorSelectionVector>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut crs_colliniar_axis: ResMut<CursorCollinearAxis>,
) {
    if let Ok((camera, camera_transform)) = camera.get_single() {
        if let Some(ref vec) = crs_vector.0 {
            let scr_vec = vec.vec();
            if scr_vec.length_squared() < 20.0 {
                return;
            }
            let x_collinearity = camera
                .world_to_viewport(camera_transform, Vec3::X)
                .map(|x_axis| scr_vec.dot(x_axis))
                .unwrap_or(f32::MAX);
            let y_collinearity = camera
                .world_to_viewport(camera_transform, Vec3::Y)
                .map(|y_axis| scr_vec.dot(y_axis))
                .unwrap_or(f32::MAX);
            let z_collinearity = camera
                .world_to_viewport(camera_transform, Vec3::Z)
                .map(|z_axis| scr_vec.dot(z_axis))
                .unwrap_or(f32::MAX);

            let (colliniarity, axis, neg_axis) = [
                (x_collinearity, Vec3::X, Vec3::NEG_X),
                (y_collinearity, Vec3::Y, Vec3::NEG_Y),
                (z_collinearity, Vec3::Z, Vec3::NEG_Z),
            ]
            .into_iter()
            .max_by(|(c1, _, _), (c2, _, _)| {
                c1.abs()
                    .partial_cmp(&c2.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
            let axis = if colliniarity.is_sign_positive() {
                axis
            } else {
                neg_axis
            };
            crs_colliniar_axis.0 = Some(axis);
        } else {
            crs_colliniar_axis.0 = None;
        }
    }
}

fn debug_axis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(
        shape::UVSphere {
            radius: 0.005,
            ..default()
        }
        .into(),
    );
    for (axis, color) in [
        (Vec3::X, Color::RED),
        (Vec3::Y, Color::GREEN),
        (Vec3::Z, Color::BLUE),
    ] {
        let material = materials.add(color.into());
        for i in 1..50 {
            commands.spawn(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_translation(axis * i as f32 * 0.05),
                ..default()
            });
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
