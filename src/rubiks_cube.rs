use bevy::{prelude::*, render::primitives::Aabb};

use crate::{
    cursor::{CollinearAxisProjection, CursorCollinearAxis, CursorRay},
    ray_extension::RayExtension,
};

const CUBE_SIDES: u32 = 3;
const CUBE_SIDE_SIZE: f32 = 0.1;
const CUBE_SPACING: f32 = 0.15;

pub struct RubiksCubePlugin;

impl Plugin for RubiksCubePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        // app.add_system(pointing_at_sub_cube);
        app.add_system(selecting_sub_cube);
        app.add_system(rotate_side);
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

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SubCube(usize);

#[derive(Debug)]
enum Rotation {
    Cw,
    Ccw,
}

#[derive(Component, Debug, Clone)]
struct RubiksCube {
    side_size: u32,
    // maps cube position to the entity
    pos_to_cube: Vec<Entity>,
    // maps entity to cube_position
    cube_to_pos: Vec<u32>,
}

impl RubiksCube {
    pub fn rotate(&mut self, cube_id: usize, cube_normal: Vec3, direction: Vec3) {
        let (axis, direction) = Self::select_axis_and_direction(cube_normal, direction);

        let selection = self.select_rotation(cube_id, axis);
        let rotated = self.rotate_indices(&selection, direction);

        let mut pos_to_cube_new = self.pos_to_cube.clone();
        for (s, r) in selection.into_iter().zip(rotated.into_iter()) {
            pos_to_cube_new[s as usize] = self.pos_to_cube[r as usize];
        }
        self.pos_to_cube = pos_to_cube_new;
    }

    pub fn rotate_indices(&self, indices: &[u32], rotation: Rotation) -> Vec<u32> {
        match rotation {
            Rotation::Ccw => (1..=self.side_size)
                .flat_map(|i| {
                    (1..=self.side_size).map(move |j| indices[(self.side_size * j - i) as usize])
                })
                .collect(),
            Rotation::Cw => (1..=self.side_size)
                .rev()
                .flat_map(|i| {
                    (1..=self.side_size)
                        .rev()
                        .map(move |j| indices[(self.side_size * j - i) as usize])
                })
                .collect(),
        }
    }

    pub fn select_rotation_entities(&self, cube_id: usize, rotation_axis: Vec3) -> Vec<Entity> {
        self.select_rotation(cube_id, rotation_axis)
            .into_iter()
            .map(|i| self.pos_to_cube[i as usize])
            .collect()
    }

    pub fn select_rotation(&self, cube_id: usize, rotation_axis: Vec3) -> Vec<u32> {
        let cube_pos = self.cube_to_pos[cube_id];
        let (x, y, z) = self.pos_to_qube_coords(cube_pos);
        if rotation_axis == Vec3::X || rotation_axis == Vec3::NEG_X {
            self.select_x_layer(x)
        } else if rotation_axis == Vec3::Y || rotation_axis == Vec3::NEG_Y {
            self.select_y_layer(y)
        } else if rotation_axis == Vec3::Z || rotation_axis == Vec3::NEG_Z {
            self.select_z_layer(z)
        } else {
            unreachable!("Axis of rotation should only be a unit base vector")
        }
    }

    fn select_axis_and_direction(normal: Vec3, direction: Vec3) -> (Vec3, Rotation) {
        let cross = normal.cross(direction);
        if cross.x.is_sign_negative() || cross.y.is_sign_negative() || cross.z.is_sign_negative() {
            (cross, Rotation::Cw)
        } else {
            (cross, Rotation::Ccw)
        }
    }

    fn corrds_to_pos(side_size: u32, x: u32, y: u32, z: u32) -> u32 {
        x * side_size * side_size + y * side_size + z
    }

    pub fn cube_corrds_to_pos(&self, x: u32, y: u32, z: u32) -> u32 {
        Self::corrds_to_pos(self.side_size, x, y, z)
    }

    pub fn pos_to_qube_coords(&self, mut cube_pos: u32) -> (u32, u32, u32) {
        let z = cube_pos % self.side_size;
        cube_pos /= self.side_size;
        let y = cube_pos % self.side_size;
        cube_pos /= self.side_size;
        let x = cube_pos % self.side_size;
        (x, y, z)
    }

    pub fn select_x_layer(&self, x: u32) -> Vec<u32> {
        (0..self.side_size)
            .flat_map(|y| (0..self.side_size).map(move |z| self.cube_corrds_to_pos(x, y, z)))
            .collect()
    }

    pub fn select_y_layer(&self, y: u32) -> Vec<u32> {
        (0..self.side_size)
            .flat_map(|x| (0..self.side_size).map(move |z| self.cube_corrds_to_pos(x, y, z)))
            .collect()
    }

    pub fn select_z_layer(&self, z: u32) -> Vec<u32> {
        (0..self.side_size)
            .flat_map(|x| (0..self.side_size).map(move |y| self.cube_corrds_to_pos(x, y, z)))
            .collect()
    }
}

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
                        let entity = builder
                            .spawn(PbrBundle {
                                mesh: sub_cube_mesh.clone(),
                                material: sub_cube_not_selected_material.clone(),
                                transform: Transform::from_xyz(
                                    offset + x as f32 * CUBE_SPACING,
                                    offset + y as f32 * CUBE_SPACING,
                                    offset + z as f32 * CUBE_SPACING,
                                ),
                                ..default()
                            })
                            .insert(SubCube(index as usize))
                            .id();
                        pos_to_cube.push(entity);
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
}

// fn pointing_at_sub_cube(
//     cursor_ray: Res<CursorRay>,
//     mut query: Query<(Entity, &Aabb, &Transform, &mut Handle<StandardMaterial>), With<SubCube>>,
//     mut currently_pointed_at_sub_cube: ResMut<CurrentlyPointedAtSubCube>,
//     mut currently_pointed_at_sub_cube_normal: ResMut<CurrentlyPointedAtSubCubeRayNormal>,
// ) {
//     // check intersections with cubes
//     let mut closest = f32::MAX;
//     let mut newly_selected = None;
//     for (entity, aabb, transform, _material) in query.iter_mut() {
//         if let Some([hit_near, _hit_far]) = cursor_ray
//             .0
//             .intersects_aabb(aabb, &transform.compute_matrix())
//         {
//             if hit_near < closest {
//                 closest = hit_near;
//                 newly_selected = Some(entity);
//             }
//         }
//     }
//
//     // sets intersection normal
//     if let Some(entity) = newly_selected {
//         if let Ok((_, aabb, transform, _)) = query.get(entity) {
//             currently_pointed_at_sub_cube_normal.0 = Some(cursor_ray.0.aabb_plane_normal(
//                 closest,
//                 aabb,
//                 &transform.compute_matrix(),
//             ));
//         }
//     }
//
//     if newly_selected != currently_pointed_at_sub_cube.0 {
//         currently_pointed_at_sub_cube.0 = newly_selected;
//     }
// }

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
            if let Ok((_, aabb, transform, _)) = query.get(entity) {
                currently_selected_sub_cube_normal.0 = Some(cursor_ray.0.aabb_plane_normal(
                    closest,
                    aabb,
                    &transform.compute_matrix(),
                ));
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
                currently_selected_sub_cube.0 = None;
                currently_selected_sub_cube_normal.0 = None;
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
                let angle = axis_projection / 100.0;
                for entity in cube_entities {
                    let (_, mut transform) = sub_cubes
                        .get_mut(entity)
                        .expect("Subcubes in rubiks cube should be in the query");
                    let rotation = if rotation_axis == Vec3::X || rotation_axis == Vec3::NEG_X {
                        Quat::from_rotation_x(-angle)
                    } else if rotation_axis == Vec3::Y || rotation_axis == Vec3::NEG_Y {
                        Quat::from_rotation_y(angle)
                    } else if rotation_axis == Vec3::Z || rotation_axis == Vec3::NEG_Z {
                        Quat::from_rotation_z(angle)
                    } else {
                        println!("Got weired rotation axis: {rotation_axis}");
                        return;
                    };
                    transform.rotate_around(Vec3::ZERO, rotation);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_rb(sides: u32) -> RubiksCube {
        let mut sub_cubes = Vec::new();
        for x in 0..sides {
            for y in 0..sides {
                for z in 0..sides {
                    let index = RubiksCube::corrds_to_pos(sides, x, y, z);
                    let entity = Entity::from_raw(index);
                    sub_cubes.push(entity);
                }
            }
        }
        RubiksCube {
            side_size: sides,
            pos_to_cube: sub_cubes,
            cube_to_pos: (0..sides.pow(3)).collect(),
        }
    }

    #[test]
    fn rb_rotate() {
        let mut rb = generate_rb(3);
        rb.rotate(0, Vec3::NEG_Y, Vec3::Z);
        let expected_cubes_pos = vec![
            6, 3, 0, 7, 4, 1, 8, 5, 2, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26,
        ]
        .into_iter()
        .map(Entity::from_raw)
        .collect::<Vec<_>>();
        assert_eq!(rb.pos_to_cube, expected_cubes_pos);

        let mut rb = generate_rb(3);
        rb.rotate(0, Vec3::NEG_X, Vec3::Z);
        let expected_cubes_pos = vec![
            18, 9, 0, 3, 4, 5, 6, 7, 8, 19, 10, 1, 12, 13, 14, 15, 16, 17, 20, 11, 2, 21, 22, 23,
            24, 25, 26,
        ]
        .into_iter()
        .map(Entity::from_raw)
        .collect::<Vec<_>>();
        assert_eq!(rb.pos_to_cube, expected_cubes_pos);

        let mut rb = generate_rb(3);
        rb.rotate(0, Vec3::NEG_X, Vec3::Y);
        let expected_cubes_pos = vec![
            18, 1, 2, 9, 4, 5, 0, 7, 8, 21, 10, 11, 12, 13, 14, 3, 16, 17, 24, 19, 20, 15, 22, 23,
            6, 25, 26,
        ]
        .into_iter()
        .map(Entity::from_raw)
        .collect::<Vec<_>>();
        assert_eq!(rb.pos_to_cube, expected_cubes_pos);
    }

    #[test]
    fn rb_rotate_indices() {
        let rb = generate_rb(3);

        let indices = rb.select_rotation(0, Vec3::X);
        let rotated = rb.rotate_indices(&indices, Rotation::Cw);
        let expected = vec![6, 3, 0, 7, 4, 1, 8, 5, 2];
        assert_eq!(rotated, expected);
        let rotated = rb.rotate_indices(&indices, Rotation::Ccw);
        let expected = vec![2, 5, 8, 1, 4, 7, 0, 3, 6];
        assert_eq!(rotated, expected);

        let indices = rb.select_rotation(0, Vec3::Y);
        let rotated = rb.rotate_indices(&indices, Rotation::Cw);
        let expected = vec![18, 9, 0, 19, 10, 1, 20, 11, 2];
        assert_eq!(rotated, expected);
        let rotated = rb.rotate_indices(&indices, Rotation::Ccw);
        let expected = vec![2, 11, 20, 1, 10, 19, 0, 9, 18];
        assert_eq!(rotated, expected);

        let indices = rb.select_rotation(0, Vec3::Z);
        let rotated = rb.rotate_indices(&indices, Rotation::Cw);
        let expected = vec![18, 9, 0, 21, 12, 3, 24, 15, 6];
        assert_eq!(rotated, expected);
        let rotated = rb.rotate_indices(&indices, Rotation::Ccw);
        let expected = vec![6, 15, 24, 3, 12, 21, 0, 9, 18];
        assert_eq!(rotated, expected);
    }

    #[test]
    fn rb_qube_coords_and_pos() {
        let sides = 3;
        let rb = generate_rb(sides);
        for x in 0..sides {
            for y in 0..sides {
                for z in 0..sides {
                    let pos = rb.cube_corrds_to_pos(x, y, z);
                    let coords = rb.pos_to_qube_coords(pos);
                    assert_eq!((x, y, z), coords);
                }
            }
        }
    }

    #[test]
    fn rb_select_layers() {
        let rb = generate_rb(3);
        for x in 0..3 {
            let offset = x * 9;
            let layer = rb.select_x_layer(x);
            let expected = (offset..offset + 9).collect::<Vec<_>>();
            assert_eq!(layer, expected)
        }
        for y in 0..3 {
            let offset = 3;
            let layer = rb.select_y_layer(y);
            let expected = vec![0, 1, 2, 9, 10, 11, 18, 19, 20]
                .into_iter()
                .map(|v| v + offset * y)
                .collect::<Vec<_>>();
            assert_eq!(layer, expected)
        }
        for z in 0..3 {
            let offset = 1;
            let layer = rb.select_z_layer(z);
            let expected = vec![0, 3, 6, 9, 12, 15, 18, 21, 24]
                .into_iter()
                .map(|v| v + offset * z)
                .collect::<Vec<_>>();
            assert_eq!(layer, expected)
        }
    }

    #[test]
    fn rb_select_rotation() {
        let rb = generate_rb(3);
        let x_entities = rb.select_rotation(0, Vec3::X);
        let x_expected = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(x_entities, x_expected);
        let y_entities = rb.select_rotation(0, Vec3::Y);
        let y_expected = vec![0, 1, 2, 9, 10, 11, 18, 19, 20];
        assert_eq!(y_entities, y_expected);
        let z_entities = rb.select_rotation(0, Vec3::Z);
        let z_expected = vec![0, 3, 6, 9, 12, 15, 18, 21, 24];
        assert_eq!(z_entities, z_expected);
    }
}
