use bevy::{prelude::*, render::primitives::Aabb};

use crate::{
    cursor::{CollinearAxisProjection, CursorCollinearAxis, CursorRay},
    ray_extension::RayExtension, rubiks_cube::RubiksCube,
};

const CUBE_SIDES: u32 = 3;
const CUBE_SIDE_SIZE: f32 = 0.1;
const CUBE_SPACING: f32 = 0.11;

pub struct RubiksCubePlugin;

impl Plugin for RubiksCubePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_system(selecting_sub_cube);
        app.add_system(rotate_side);
        app.add_system(stop_rotation);
    }
}

#[derive(Resource, Debug, Default)]
struct SubCubeMaterials {
    selected: Handle<StandardMaterial>,
    pointed: Handle<StandardMaterial>,
    not_selected: Handle<StandardMaterial>,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct CurrentlyPointedAtSubCube(Option<Entity>);

#[derive(Resource, Debug, Default, Clone, Copy)]
struct CurrentlyPointedAtSubCubeRayNormal(Option<Vec3>);

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct CurrentlySelectedSubCube(Option<Entity>);

#[derive(Resource, Debug, Default, Clone, Copy)]
struct CurrentlySelectedSubCubeRayNormal(Option<Vec3>);

#[derive(Resource, Debug, Default, Clone, Copy)]
struct RotationAngle(f32);

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SubCube(usize);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sub_cube_mesh = meshes.add(Mesh::from(shape::Cube {
        size: CUBE_SIDE_SIZE,
    }));
    let sub_cube_selected_material = materials.add(Color::ORANGE.into());
    let sub_cube_pointed_material = materials.add(Color::GREEN.into());
    let sub_cube_not_selected_material = materials.add(Color::WHITE.into());
    let mut pos_to_cube = Vec::new();
    commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::default(),
            ComputedVisibility::default(),
        ))
        .with_children(|builder| {
            let offset = match CUBE_SIDES % 2 {
                0 => -CUBE_SPACING / 2.0 - (CUBE_SIDES - 1) as f32 / 2.0 * CUBE_SPACING,
                1 => -(CUBE_SIDES as i32 / 2) as f32 * CUBE_SPACING,
                _ => unreachable!(),
            };
            for x in 0..CUBE_SIDES {
                for y in 0..CUBE_SIDES {
                    for z in 0..CUBE_SIDES {
                        // basically xyz represents a number in CUBE_SIDES base system and
                        // index is dec representation of it
                        // so this index can be used as just id of a cube
                        // and as a mapping to the position of the qube
                        let index = RubiksCube::corrds_to_pos(CUBE_SIDES, x, y, z);
                        let material = materials.add(
                            Color::Rgba {
                                red: index as f32 / CUBE_SIDES.pow(3) as f32,
                                green: index as f32 / CUBE_SIDES.pow(3) as f32,
                                blue: index as f32 / CUBE_SIDES.pow(3) as f32,
                                alpha: 1.0,
                            }
                            .into(),
                        );
                        let entity = builder
                            .spawn(PbrBundle {
                                mesh: sub_cube_mesh.clone(),
                                // material: sub_cube_not_selected_material.clone(),
                                material,
                                transform: Transform::from_xyz(
                                    offset + x as f32 * CUBE_SPACING,
                                    offset + y as f32 * CUBE_SPACING,
                                    offset + z as f32 * CUBE_SPACING,
                                ),
                                ..default()
                            })
                            .insert(SubCube(index as usize))
                            .id();
                        pos_to_cube.push((entity, index as usize));
                    }
                }
            }
        })
        .insert(RubiksCube {
            side_size: CUBE_SIDES,
            pos_to_cube,
            cube_to_pos: (0..CUBE_SIDES.pow(3)).collect(),
        });

    commands.insert_resource(SubCubeMaterials {
        selected: sub_cube_selected_material,
        pointed: sub_cube_pointed_material,
        not_selected: sub_cube_not_selected_material,
    });

    commands.insert_resource(CurrentlyPointedAtSubCube::default());
    commands.insert_resource(CurrentlyPointedAtSubCubeRayNormal::default());
    commands.insert_resource(CurrentlySelectedSubCube::default());
    commands.insert_resource(CurrentlySelectedSubCubeRayNormal::default());
    commands.insert_resource(RotationAngle::default());
}

fn selecting_sub_cube(
    mouse_input: Res<Input<MouseButton>>,
    cursor_ray: Res<CursorRay>,
    sub_cube_materials: Res<SubCubeMaterials>,
    mut query: Query<(Entity, &Aabb, &Transform, &mut Handle<StandardMaterial>), With<SubCube>>,
    mut currently_selected_sub_cube: ResMut<CurrentlySelectedSubCube>,
    mut currently_selected_sub_cube_normal: ResMut<CurrentlySelectedSubCubeRayNormal>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let mut closest = f32::MAX;
        let mut newly_selected = None;
        for (entity, aabb, transform, _) in query.iter_mut() {
            if let Some([hit_near, _hit_far]) = cursor_ray
                .0
                .intersects_aabb(aabb, &transform.compute_matrix())
            {
                if hit_near < closest {
                    closest = hit_near;
                    newly_selected = Some(entity);
                }
            }
        }

        // sets intersection normal
        if let Some(entity) = newly_selected {
            if let Ok((_, aabb, _, _)) = query.get(entity) {
                currently_selected_sub_cube_normal.0 =
                    Some(cursor_ray.0.aabb_plane_normal(closest, aabb));
            }
        }

        currently_selected_sub_cube.0 = newly_selected;

        // color selected cube
        if let Some(entity) = currently_selected_sub_cube.0 {
            if let Ok(mut material) = query.get_component_mut::<Handle<StandardMaterial>>(entity) {
                *material = sub_cube_materials.selected.clone();
            }
        }
    } else if mouse_input.just_released(MouseButton::Left) {
        // remove color from perviously selected cube
        if let Some(entity) = currently_selected_sub_cube.0 {
            if let Ok((_, _, _, mut sub_cube_material)) = query.get_mut(entity) {
                *sub_cube_material = sub_cube_materials.not_selected.clone();
            }
        }
    }
}

fn rotate_side(
    rubiks_cube: Query<&RubiksCube>,
    currently_selected_sub_cube: Res<CurrentlySelectedSubCube>,
    currently_selected_sub_cube_normal: Res<CurrentlySelectedSubCubeRayNormal>,
    cursor_collinear_axis: Res<CursorCollinearAxis>,
    collinear_axis_projection: Res<CollinearAxisProjection>,
    mut sub_cubes: Query<(&SubCube, &mut Transform)>,
    mut rotation_angle: ResMut<RotationAngle>,
) {
    if let Ok(rb) = rubiks_cube.get_single() {
        if let (
            Some(selected_cube),
            Some(selected_sub_cube_normal),
            Some(direction),
            Some(axis_projection),
        ) = (
            currently_selected_sub_cube.0,
            currently_selected_sub_cube_normal.0,
            cursor_collinear_axis.0,
            collinear_axis_projection.0,
        ) {
            let (rotation_axis, _) =
                RubiksCube::select_axis_and_direction(selected_sub_cube_normal, direction);
            if let Ok(sub_cube) = sub_cubes.get_component::<SubCube>(selected_cube) {
                let cube_entities = rb.select_rotation_entities(sub_cube.0, rotation_axis);
                let angle = (axis_projection * 50.0).clamp(-1.0, 1.0) * std::f32::consts::FRAC_PI_2;
                let diff = angle - rotation_angle.0;
                rotation_angle.0 = angle;
                for entity in cube_entities {
                    let (_, mut transform) = sub_cubes
                        .get_mut(entity)
                        .expect("Subcubes in rubiks cube should be in the query");
                    let rotation = if rotation_axis == Vec3::X {
                        Quat::from_rotation_x(diff)
                    } else if rotation_axis == Vec3::NEG_X {
                        Quat::from_rotation_x(-diff)
                    } else if rotation_axis == Vec3::Y {
                        Quat::from_rotation_y(diff)
                    } else if rotation_axis == Vec3::NEG_Y {
                        Quat::from_rotation_y(-diff)
                    } else if rotation_axis == Vec3::Z {
                        Quat::from_rotation_z(diff)
                    } else if rotation_axis == Vec3::NEG_Z {
                        Quat::from_rotation_z(-diff)
                    } else {
                        println!("Got weired rotation axis: {rotation_axis}");
                        return;
                    };
                    transform.rotate_around(Vec3::ZERO, rotation);
                }
            }
        } else {
            rotation_angle.0 = 0.0;
        }
    }
}

fn stop_rotation(
    mouse_input: Res<Input<MouseButton>>,
    rotation_angle: Res<RotationAngle>,
    mut currently_selected_sub_cube: ResMut<CurrentlySelectedSubCube>,
    mut currently_selected_sub_cube_normal: ResMut<CurrentlySelectedSubCubeRayNormal>,
    mut cursor_collinear_axis: ResMut<CursorCollinearAxis>,
    mut collinear_axis_projection: ResMut<CollinearAxisProjection>,
    mut rubiks_cube: Query<&mut RubiksCube>,
    mut sub_cubes: Query<(&SubCube, &mut Transform)>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        if let (Ok(mut rb), Some(selected_cube), Some(selected_sub_cube_normal), Some(direction)) = (
            rubiks_cube.get_single_mut(),
            currently_selected_sub_cube.0,
            currently_selected_sub_cube_normal.0,
            cursor_collinear_axis.0,
        ) {
            let (rotation_axis, _) =
                RubiksCube::select_axis_and_direction(selected_sub_cube_normal, direction);
            let sub_cube = sub_cubes.get_component::<SubCube>(selected_cube).unwrap();
            let cube_entities = rb.select_rotation_entities(sub_cube.0, rotation_axis);
            let angle = if rotation_angle.0.abs() > std::f32::consts::FRAC_PI_4 {
                let angle = if rotation_angle.0.is_sign_positive() {
                    std::f32::consts::FRAC_PI_2 - rotation_angle.0
                } else {
                    -std::f32::consts::FRAC_PI_2 + rotation_angle.0
                };
                rb.rotate(sub_cube.0, selected_sub_cube_normal, direction);
                angle
            } else {
                -rotation_angle.0
            };
            for entity in cube_entities {
                let (_, mut transform) = sub_cubes
                    .get_mut(entity)
                    .expect("Subcubes in rubiks cube should be in the query");
                let rotation = if rotation_axis == Vec3::X {
                    Quat::from_rotation_x(angle)
                } else if rotation_axis == Vec3::NEG_X {
                    Quat::from_rotation_x(-angle)
                } else if rotation_axis == Vec3::Y {
                    Quat::from_rotation_y(angle)
                } else if rotation_axis == Vec3::NEG_Y {
                    Quat::from_rotation_y(-angle)
                } else if rotation_axis == Vec3::Z {
                    Quat::from_rotation_z(angle)
                } else if rotation_axis == Vec3::NEG_Z {
                    Quat::from_rotation_z(-angle)
                } else {
                    println!("Got weired rotation axis: {rotation_axis}");
                    return;
                };
                transform.rotate_around(Vec3::ZERO, rotation);
            }
        }
        currently_selected_sub_cube.0 = None;
        currently_selected_sub_cube_normal.0 = None;
        cursor_collinear_axis.0 = None;
        collinear_axis_projection.0 = None;
    }
}
