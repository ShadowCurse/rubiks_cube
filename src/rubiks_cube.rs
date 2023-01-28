use std::fmt::Display;

use bevy::prelude::{Component, Entity, Vec3};

#[derive(Debug, PartialEq, Eq)]
pub enum Rotation {
    Cw,
    Ccw,
}

#[derive(Component, Debug, Clone)]
pub struct RubiksCube {
    pub side_size: u32,
    // maps cube position to the entity
    pub pos_to_cube: Vec<(Entity, usize)>,
    // maps entity to cube_position
    pub cube_to_pos: Vec<u32>,
}

impl RubiksCube {
    pub fn is_solved(&self) -> bool {
        self.pos_to_cube.is_sorted_by_key(|(_, i)| i)
    }

    pub fn rotate(&mut self, cube_id: usize, cube_normal: Vec3, direction: Vec3) {
        let (axis, rotation) = Self::select_axis_and_rotation(cube_normal, direction);

        let selection = self.select_rotation(cube_id, axis);
        let rotated = self.rotate_indices(&selection, rotation);

        let mut pos_to_cube_new = self.pos_to_cube.clone();
        let mut cube_to_pos_new = self.cube_to_pos.clone();
        for (s, r) in selection.iter().zip(rotated.iter()) {
            pos_to_cube_new[*s as usize] = self.pos_to_cube[*r as usize];
        }
        for s in selection {
            cube_to_pos_new[pos_to_cube_new[s as usize].1] = s;
        }
        self.pos_to_cube = pos_to_cube_new;
        self.cube_to_pos = cube_to_pos_new;
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
            .map(|i| self.pos_to_cube[i as usize].0)
            .collect()
    }

    pub fn select_rotation(&self, cube_id: usize, rotation_axis: Vec3) -> Vec<u32> {
        let cube_pos = self.cube_to_pos[cube_id];
        let (x, y, z) = self.pos_to_qube_coords(cube_pos);
        if rotation_axis == Vec3::X {
            self.select_x_layer(x)
        } else if rotation_axis == Vec3::Y {
            self.select_y_layer(y)
        } else if rotation_axis == Vec3::Z {
            self.select_z_layer(z)
        } else {
            unreachable!("Axis of rotation should only be a unit base vector: {rotation_axis}");
        }
    }

    pub fn select_axis_and_rotation(normal: Vec3, direction: Vec3) -> (Vec3, Rotation) {
        let cross = normal.cross(direction);
        if cross.x < 0.0 || cross.y < 0.0 || cross.z < 0.0 {
            (cross.abs(), Rotation::Cw)
        } else {
            (cross.abs(), Rotation::Ccw)
        }
    }

    pub fn corrds_to_pos(side_size: u32, x: u32, y: u32, z: u32) -> u32 {
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
            .rev()
            .flat_map(|x| (0..self.side_size).map(move |z| self.cube_corrds_to_pos(x, y, z)))
            .collect()
    }

    pub fn select_z_layer(&self, z: u32) -> Vec<u32> {
        (0..self.side_size)
            .flat_map(|y| {
                (0..self.side_size)
                    .rev()
                    .map(move |x| self.cube_corrds_to_pos(x, y, z))
            })
            .collect()
    }
}

impl Display for RubiksCube {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--->z")?;
        writeln!(f, "|")?;
        writeln!(f, "|")?;
        writeln!(f, "|")?;
        writeln!(f, "\u{2193}")?;
        writeln!(f, "y")?;
        writeln!(f, "|")?;
        writeln!(f, "|")?;
        writeln!(f, "|")?;
        writeln!(f, "\u{2193}")?;
        writeln!(f, "x")?;
        for x in 0..self.side_size {
            for y in 0..self.side_size {
                for z in 0..self.side_size {
                    let index = self.cube_corrds_to_pos(x, y, z);
                    write!(f, " {:?} ", self.cube_to_pos[index as usize])?;
                }
                writeln!(f)?;
            }
            writeln!(f)?;
        }
        Ok(())
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
                    sub_cubes.push((entity, index as usize));
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
    fn rb_select_axis_and_rotation() {
        // X
        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::X, Vec3::Y);
        assert_eq!(axis, Vec3::Z);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::X, Vec3::NEG_Y);
        assert_eq!(axis, Vec3::Z);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::X, Vec3::Z);
        assert_eq!(axis, Vec3::Y);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::X, Vec3::NEG_Z);
        assert_eq!(axis, Vec3::Y);
        assert_eq!(rotation, Rotation::Ccw);

        // NEG_X
        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_X, Vec3::Y);
        assert_eq!(axis, Vec3::Z);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_X, Vec3::NEG_Y);
        assert_eq!(axis, Vec3::Z);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_X, Vec3::Z);
        assert_eq!(axis, Vec3::Y);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_X, Vec3::NEG_Z);
        assert_eq!(axis, Vec3::Y);
        assert_eq!(rotation, Rotation::Cw);

        // Y
        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::Y, Vec3::X);
        assert_eq!(axis, Vec3::Z);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::Y, Vec3::NEG_X);
        assert_eq!(axis, Vec3::Z);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::Y, Vec3::Z);
        assert_eq!(axis, Vec3::X);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::Y, Vec3::NEG_Z);
        assert_eq!(axis, Vec3::X);
        assert_eq!(rotation, Rotation::Cw);

        // NEG_Y
        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_Y, Vec3::X);
        assert_eq!(axis, Vec3::Z);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_Y, Vec3::NEG_X);
        assert_eq!(axis, Vec3::Z);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_Y, Vec3::Z);
        assert_eq!(axis, Vec3::X);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_Y, Vec3::NEG_Z);
        assert_eq!(axis, Vec3::X);
        assert_eq!(rotation, Rotation::Ccw);

        // Z
        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::Z, Vec3::X);
        assert_eq!(axis, Vec3::Y);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::Z, Vec3::NEG_X);
        assert_eq!(axis, Vec3::Y);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::Z, Vec3::Y);
        assert_eq!(axis, Vec3::X);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::Z, Vec3::NEG_Y);
        assert_eq!(axis, Vec3::X);
        assert_eq!(rotation, Rotation::Ccw);

        // NEG_Z
        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_Z, Vec3::X);
        assert_eq!(axis, Vec3::Y);
        assert_eq!(rotation, Rotation::Cw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_Z, Vec3::NEG_X);
        assert_eq!(axis, Vec3::Y);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_Z, Vec3::Y);
        assert_eq!(axis, Vec3::X);
        assert_eq!(rotation, Rotation::Ccw);

        let (axis, rotation) = RubiksCube::select_axis_and_rotation(Vec3::NEG_Z, Vec3::NEG_Y);
        assert_eq!(axis, Vec3::X);
        assert_eq!(rotation, Rotation::Cw);
    }

    #[test]
    fn rb_rotate_single() {
        let mut rb = generate_rb(3);
        rb.rotate(0, Vec3::NEG_Y, Vec3::Z);
        let expected_cubes_pos = vec![
            6, 3, 0, 7, 4, 1, 8, 5, 2, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26,
        ]
        .into_iter()
        .map(|c| (Entity::from_raw(c), c as usize))
        .collect::<Vec<_>>();
        assert_eq!(rb.pos_to_cube, expected_cubes_pos);

        let mut rb = generate_rb(3);
        rb.rotate(0, Vec3::NEG_X, Vec3::Z);
        let expected_cubes_pos = vec![
            18, 9, 0, 3, 4, 5, 6, 7, 8, 19, 10, 1, 12, 13, 14, 15, 16, 17, 20, 11, 2, 21, 22, 23,
            24, 25, 26,
        ]
        .into_iter()
        .map(|c| (Entity::from_raw(c), c as usize))
        .collect::<Vec<_>>();
        assert_eq!(rb.pos_to_cube, expected_cubes_pos);

        let mut rb = generate_rb(3);
        rb.rotate(0, Vec3::NEG_X, Vec3::Y);
        let expected_cubes_pos = vec![
            18, 1, 2, 9, 4, 5, 0, 7, 8, 21, 10, 11, 12, 13, 14, 3, 16, 17, 24, 19, 20, 15, 22, 23,
            6, 25, 26,
        ]
        .into_iter()
        .map(|c| (Entity::from_raw(c), c as usize))
        .collect::<Vec<_>>();
        assert_eq!(rb.pos_to_cube, expected_cubes_pos);
    }

    #[test]
    fn rb_rotate_multiple() {
        let mut rb = generate_rb(3);
        rb.rotate(0, Vec3::NEG_Y, Vec3::Z);
        let expected_cubes_pos = vec![
            6, 3, 0, 7, 4, 1, 8, 5, 2, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26,
        ]
        .into_iter()
        .map(|c| (Entity::from_raw(c), c as usize))
        .collect::<Vec<_>>();
        assert_eq!(rb.pos_to_cube, expected_cubes_pos);

        rb.rotate(6, Vec3::NEG_X, Vec3::Z);
        let expected_cubes_pos = vec![
            18, 9, 6, 7, 4, 1, 8, 5, 2, 19, 10, 3, 12, 13, 14, 15, 16, 17, 20, 11, 0, 21, 22, 23,
            24, 25, 26,
        ]
        .into_iter()
        .map(|c| (Entity::from_raw(c), c as usize))
        .collect::<Vec<_>>();
        assert_eq!(rb.pos_to_cube, expected_cubes_pos);

        rb.rotate(20, Vec3::NEG_Y, Vec3::Z);
        let expected_cubes_pos = vec![
            18, 9, 6, 7, 4, 1, 8, 5, 2, 19, 10, 3, 12, 13, 14, 15, 16, 17, 24, 21, 20, 25, 22, 11,
            26, 23, 0,
        ]
        .into_iter()
        .map(|c| (Entity::from_raw(c), c as usize))
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
        let expected = vec![0, 9, 18, 1, 10, 19, 2, 11, 20];
        assert_eq!(rotated, expected);
        let rotated = rb.rotate_indices(&indices, Rotation::Ccw);
        let expected = vec![20, 11, 2, 19, 10, 1, 18, 9, 0];
        assert_eq!(rotated, expected);

        let indices = rb.select_rotation(0, Vec3::Z);
        let rotated = rb.rotate_indices(&indices, Rotation::Cw);
        let expected = vec![24, 21, 18, 15, 12, 9, 6, 3, 0];
        assert_eq!(rotated, expected);
        let rotated = rb.rotate_indices(&indices, Rotation::Ccw);
        let expected = vec![0, 3, 6, 9, 12, 15, 18, 21, 24];
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
            let expected = vec![18, 19, 20, 9, 10, 11, 0, 1, 2]
                .into_iter()
                .map(|v| v + offset * y)
                .collect::<Vec<_>>();
            assert_eq!(layer, expected)
        }
        for z in 0..3 {
            let offset = 1;
            let layer = rb.select_z_layer(z);
            let expected = vec![18, 9, 0, 21, 12, 3, 24, 15, 6]
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
        let y_expected = vec![18, 19, 20, 9, 10, 11, 0, 1, 2];
        assert_eq!(y_entities, y_expected);
        let z_entities = rb.select_rotation(0, Vec3::Z);
        let z_expected = vec![18, 9, 0, 21, 12, 3, 24, 15, 6];
        assert_eq!(z_entities, z_expected);
    }
}
