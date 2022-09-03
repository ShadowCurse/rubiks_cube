use bevy::{prelude::*, render::primitives::Aabb};

use crate::cursor::CursorRay;

const CUBE_SIDES: u32 = 4;
const CUBE_SIDE_SIZE: f32 = 0.1;
const CUBE_SPACING: f32 = 0.15;

pub struct RubiksCubePlugin;

impl Plugin for RubiksCubePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_system(sub_cube_selection);
    }
}

#[derive(Resource, Debug, Default)]
struct SubCubeMaterials {
    selected: Handle<StandardMaterial>,
    not_selected: Handle<StandardMaterial>,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct CurrentlySelectedSubCube(Option<Entity>);

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SubCube(u32);

#[derive(Component, Debug, Clone)]
struct RubiksCube {
    side_size: u32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sub_cube_mesh = meshes.add(Mesh::from(shape::Cube {
        size: CUBE_SIDE_SIZE as f32,
    }));
    let sub_cube_selected_material = materials.add(Color::GREEN.into());
    let sub_cube_not_selected_material = materials.add(Color::WHITE.into());
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
                        // skipping insides
                        if x > 0
                            && x < CUBE_SIDES - 1
                            && y > 0
                            && y < CUBE_SIDES - 1
                            && z > 0
                            && z < CUBE_SIDES - 1
                        {
                            continue;
                        }
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
                            .insert(SubCube(index))
                            .id();
                    }
                }
            }
        })
        .insert(RubiksCube {
            side_size: CUBE_SIDES,
        });

    commands.insert_resource(SubCubeMaterials {
        selected: sub_cube_selected_material,
        not_selected: sub_cube_not_selected_material,
    });

    commands.insert_resource(CurrentlySelectedSubCube::default());
}

fn sub_cube_selection(
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
    mut currently_selected_sub_cube: ResMut<CurrentlySelectedSubCube>,
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
                *material = sub_cube_materials.selected.clone();
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
