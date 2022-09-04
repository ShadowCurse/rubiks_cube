use bevy::{prelude::*, render::primitives::Aabb};

use crate::cursor::CursorRay;

const CUBE_SIDES: u32 = 4;
const CUBE_SIDE_SIZE: f32 = 0.1;
const CUBE_SPACING: f32 = 0.15;

pub struct RubiksCubePlugin;

impl Plugin for RubiksCubePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_system(pointing_at_sub_cube);
        app.add_system(selecting_sub_cube);
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

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct CurrentlySelectedSubCube(Option<Entity>);

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SubCube(usize);

#[derive(Component, Debug, Clone)]
struct RubiksCube {
    side_size: u32,
    cubes: Vec<Entity>,
}

impl RubiksCube {
    pub fn select_vertical(&self, index: usize) -> Vec<Entity> {
        let layer_size = self.side_size.pow(2);
        let vert_layer = self.vertical_layer(index);
        ((vert_layer * layer_size)..((vert_layer + 1) * layer_size))
            .map(|i| self.cubes[i as usize])
            .collect()
    }

    pub fn select_horizontal(&self, index: usize) -> Vec<Entity> {
        let hor_layer = self.horizontal_layer(index);
        (0..self.side_size)
            .flat_map(|vert_layer| {
                ((hor_layer * self.side_size) + vert_layer * self.side_size.pow(2)
                    ..(hor_layer * self.side_size + self.side_size)
                        + vert_layer * self.side_size.pow(2))
                    .map(|i| self.cubes[i as usize])
            })
            .collect()
    }

    /// returns vertical cube layer from 0 to side_size
    fn vertical_layer(&self, index: usize) -> u32 {
        let layer_size = self.side_size.pow(2);
        for i in 0..self.side_size {
            if (index as u32) <= layer_size * (i + 1) {
                return i;
            }
        }
        unreachable!()
    }

    /// returns horizontal cube layer from 0 to side_size
    fn horizontal_layer(&self, index: usize) -> u32 {
        let vert_layer = self.vertical_layer(index);
        for i in 0..self.side_size {
            if (i * self.side_size + 1) + vert_layer * self.side_size.pow(2) <= (index as u32)
                && (index as u32)
                    <= (i * self.side_size + self.side_size) + vert_layer * self.side_size.pow(2)
            {
                return i;
            }
        }
        unreachable!()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sub_cube_mesh = meshes.add(Mesh::from(shape::Cube {
        size: CUBE_SIDE_SIZE as f32,
    }));
    let sub_cube_selected_material = materials.add(Color::ORANGE.into());
    let sub_cube_pointed_material = materials.add(Color::GREEN.into());
    let sub_cube_not_selected_material = materials.add(Color::WHITE.into());
    let mut sub_cubes = Vec::new();
    commands
        .spawn_bundle((
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
                        let index = CUBE_SIDES * CUBE_SIDES * x + CUBE_SIDES * y + z + 1;
                        let entity = builder
                            .spawn_bundle(PbrBundle {
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
                        sub_cubes.push(entity);
                    }
                }
            }
        })
        .insert(RubiksCube {
            side_size: CUBE_SIDES,
            cubes: sub_cubes,
        });

    commands.insert_resource(SubCubeMaterials {
        selected: sub_cube_selected_material,
        pointed: sub_cube_pointed_material,
        not_selected: sub_cube_not_selected_material,
    });

    commands.insert_resource(CurrentlyPointedAtSubCube::default());
    commands.insert_resource(CurrentlySelectedSubCube::default());
}

fn pointing_at_sub_cube(
    cursor_ray: Res<CursorRay>,
    sub_cube_materials: Res<SubCubeMaterials>,
    mut query: Query<
        (
            Entity,
            &Aabb,
            &GlobalTransform,
            &mut Handle<StandardMaterial>,
        ),
        With<SubCube>,
    >,
    mut currently_selected_sub_cube: ResMut<CurrentlyPointedAtSubCube>,
) {
    let mut closest = f32::MAX;
    let mut newly_selected = None;
    for (entity, aabb, transform, _material) in query.iter_mut() {
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
    if newly_selected != currently_selected_sub_cube.0 {
        if let Some(entity) = newly_selected {
            if let Ok(mut material) = query.get_component_mut::<Handle<StandardMaterial>>(entity) {
                *material = sub_cube_materials.pointed.clone();
            }
        }

        if let Some(currently_selected) = currently_selected_sub_cube.0 {
            if let Ok(mut material) =
                query.get_component_mut::<Handle<StandardMaterial>>(currently_selected)
            {
                *material = sub_cube_materials.not_selected.clone();
            }
        }

        currently_selected_sub_cube.0 = newly_selected;
    }
}

fn selecting_sub_cube(
    key_input: Res<Input<KeyCode>>,
    sub_cube_materials: Res<SubCubeMaterials>,
    currently_pointed_at_sub_cube: Res<CurrentlyPointedAtSubCube>,
    mut sub_cubes: Query<&mut Handle<StandardMaterial>, With<SubCube>>,
    mut currently_selected_sub_cube: ResMut<CurrentlySelectedSubCube>,
) {
    if key_input.just_pressed(KeyCode::Space) {
        if let Some(entity) = currently_selected_sub_cube.0 {
            if let Ok(mut sub_cube_material) = sub_cubes.get_mut(entity) {
                *sub_cube_material = sub_cube_materials.not_selected.clone();
                currently_selected_sub_cube.0 = None;
            }
        }
        if let Some(entity) = currently_pointed_at_sub_cube.0 {
            if let Ok(mut sub_cube_material) = sub_cubes.get_mut(entity) {
                *sub_cube_material = sub_cube_materials.selected.clone();
                currently_selected_sub_cube.0 = currently_pointed_at_sub_cube.0;
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
                    let index = sides * sides * x + sides * y + z + 1;
                    let entity = Entity::from_raw(index);
                    sub_cubes.push(entity);
                }
            }
        }
        RubiksCube {
            side_size: sides,
            cubes: sub_cubes,
        }
    }

    #[test]
    fn vertical_layer() {
        let rb = generate_rb(3);
        for i in 1..10 {
            assert_eq!(rb.vertical_layer(i), 0);
        }
        for i in 10..19 {
            assert_eq!(rb.vertical_layer(i), 1);
        }
        for i in 19..28 {
            assert_eq!(rb.vertical_layer(i), 2);
        }
    }

    #[test]
    fn horizontal_layer() {
        let rb = generate_rb(3);
        let layers = [
            ([1, 2, 3], 0),
            ([4, 5, 6], 1),
            ([7, 8, 9], 2),
            ([10, 11, 12], 0),
            ([13, 14, 15], 1),
            ([16, 17, 18], 2),
            ([19, 20, 21], 0),
            ([22, 23, 24], 1),
            ([25, 26, 27], 2),
        ];
        for (cubes, layer) in layers {
            for c in cubes {
                assert_eq!(rb.horizontal_layer(c), layer);
            }
        }
    }

    #[test]
    fn select_vertical_layer() {
        let rb = generate_rb(3);
        for i in 1..10 {
            assert_eq!(
                rb.select_vertical(i),
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
                    .into_iter()
                    .map(Entity::from_raw)
                    .collect::<Vec<_>>()
            );
        }
        for i in 10..19 {
            assert_eq!(
                rb.select_vertical(i),
                vec![10, 11, 12, 13, 14, 15, 16, 17, 18]
                    .into_iter()
                    .map(Entity::from_raw)
                    .collect::<Vec<_>>()
            );
        }
        for i in 19..28 {
            assert_eq!(
                rb.select_vertical(i),
                vec![19, 20, 21, 22, 23, 24, 25, 26, 27]
                    .into_iter()
                    .map(Entity::from_raw)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn select_horizontal_layer() {
        let rb = generate_rb(3);
        let layers = [
            [1, 2, 3, 10, 11, 12, 19, 20, 21],
            [4, 5, 6, 13, 14, 15, 22, 23, 24],
            [7, 8, 9, 16, 17, 18, 25, 26, 27],
        ];
        for layer in layers {
            let entity_layer = layer
                .iter()
                .cloned()
                .map(Entity::from_raw)
                .collect::<Vec<_>>();
            for l in layer {
                assert_eq!(rb.select_horizontal(l as usize), entity_layer);
            }
        }
    }
}
