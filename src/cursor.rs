use bevy::prelude::*;

use crate::{rubiks_cube_plugin::CurrentlySelectedSubCubeRayNormal, GameStates};

pub struct CursorRayPlugin;

impl Plugin for CursorRayPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameStates::InGame).with_system(init));
        app.add_system_set(
            SystemSet::on_update(GameStates::InGame)
                .with_system(world_cursor_system)
                .with_system(cursor_selection_vector)
                .with_system(selection_vector_colliniar_axis)
                .with_system(projection_on_collinear_axis),
        );
        app.add_system_set(SystemSet::on_exit(GameStates::InGame).with_system(deinit));
    }
}

#[derive(Resource, Debug, Default)]
pub struct CursorRay(pub Ray);

#[derive(Debug, Default, Clone, Copy)]
pub struct ScreenVector {
    start: Vec2,
    end: Vec2,
}

#[derive(Resource, Debug, Default)]
pub struct CursorSelectionVector(pub Option<ScreenVector>);

#[derive(Resource, Debug, Default)]
pub struct CursorCollinearAxis(pub Option<Vec3>);

#[derive(Resource, Debug, Default)]
pub struct CollinearAxisProjection(pub Option<f32>);

fn init(mut commands: Commands) {
    commands.insert_resource(CursorRay::default());
    commands.insert_resource(CursorSelectionVector::default());
    commands.insert_resource(CursorCollinearAxis::default());
    commands.insert_resource(CollinearAxisProjection::default());
}

fn deinit(mut commands: Commands) {
    commands.remove_resource::<CursorRay>();
    commands.remove_resource::<CursorSelectionVector>();
    commands.remove_resource::<CursorCollinearAxis>();
    commands.remove_resource::<CollinearAxisProjection>();
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
            if let Some(ray) = camera.viewport_to_world(camera_transform, screen_pos) {
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
    }
}

// selecte the axis that is collinear to the cursor crs_vector
// excludes the axis that is the same as the normal
fn selection_vector_colliniar_axis(
    crs_vector: Res<CursorSelectionVector>,
    camera: Query<(&Camera, &GlobalTransform)>,
    currently_selected_sub_cube_normal: Res<CurrentlySelectedSubCubeRayNormal>,
    mut crs_colliniar_axis: ResMut<CursorCollinearAxis>,
) {
    if let (Ok((camera, camera_transform)), Some(normal), Some(vec), None) = (
        camera.get_single(),
        currently_selected_sub_cube_normal.0,
        crs_vector.0,
        crs_colliniar_axis.0,
    ) {
        if let (Some(ray_start), Some(ray_end)) = (
            camera.viewport_to_world(camera_transform, vec.start),
            camera.viewport_to_world(camera_transform, vec.end),
        ) {
            let vec = ray_end.origin - ray_start.origin;
            // have some room for the mouse
            if vec.length_squared() < 1e-8 {
                return;
            }

            let vec = vec.normalize();
            let x_collinearity = vec.dot(Vec3::X);
            let y_collinearity = vec.dot(Vec3::Y);
            let z_collinearity = vec.dot(Vec3::Z);

            let (colliniarity, axis, neg_axis) = [
                (x_collinearity, Vec3::X, Vec3::NEG_X),
                (y_collinearity, Vec3::Y, Vec3::NEG_Y),
                (z_collinearity, Vec3::Z, Vec3::NEG_Z),
            ]
            .into_iter()
            .filter(|(_, axis, neg_axis)| axis != &normal && neg_axis != &normal)
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

fn projection_on_collinear_axis(
    camera: Query<(&Camera, &GlobalTransform)>,
    crs_vector: Res<CursorSelectionVector>,
    crs_colliniar_axis: Res<CursorCollinearAxis>,
    mut colliniar_axis_projection: ResMut<CollinearAxisProjection>,
) {
    if let (Some(vec), Some(axis), Ok((camera, camera_transform))) =
        (crs_vector.0, crs_colliniar_axis.0, camera.get_single())
    {
        if let (Some(ray_start), Some(ray_end)) = (
            camera.viewport_to_world(camera_transform, vec.start),
            camera.viewport_to_world(camera_transform, vec.end),
        ) {
            let vec = ray_end.origin - ray_start.origin;
            let projection = vec.dot(axis);
            colliniar_axis_projection.0 = Some(projection);
        }
    } else {
        colliniar_axis_projection.0 = None;
    }
}

// fn debug_axis(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let mesh = meshes.add(
//         shape::UVSphere {
//             radius: 0.005,
//             ..default()
//         }
//         .into(),
//     );
//     for (axis, color) in [
//         (Vec3::X, Color::RED),
//         (Vec3::Y, Color::GREEN),
//         (Vec3::Z, Color::BLUE),
//     ] {
//         let material = materials.add(color.into());
//         for i in 1..50 {
//             commands.spawn(PbrBundle {
//                 mesh: mesh.clone(),
//                 material: material.clone(),
//                 transform: Transform::from_translation(axis * i as f32 * 0.05),
//                 ..default()
//             });
//         }
//     }
// }
