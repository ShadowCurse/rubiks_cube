use crate::ray::Ray;
use bevy::prelude::*;

pub struct CursorRayPlugin;

impl Plugin for CursorRayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorRay::default());
        app.add_system(world_cursor_system);
    }
}

pub type CursorRay = Ray;

fn world_cursor_system(
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut crs: ResMut<CursorRay>,
) {
    if let Ok((camera, camera_transform)) = camera.get_single() {
        let window = windows.get_primary().unwrap();
        if let Some(screen_pos) = window.cursor_position() {
            if let Some(ray) = CursorRay::from_screenspace(screen_pos, camera, camera_transform) {
                *crs = ray;
            }
        }
    }
}
