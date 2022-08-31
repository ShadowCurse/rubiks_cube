use bevy::{prelude::*, math::Vec3A, render::primitives::Aabb};

#[derive(Resource, Debug, Default, Clone)]
pub struct Ray {
    origin: Vec3A,
    direction: Vec3A,
}

impl Ray {
    pub fn from_screenspace(
        cursor_pos_screen: Vec2,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<Self> {
        let view = camera_transform.compute_matrix();

        let (viewport_min, viewport_max) = camera.logical_viewport_rect()?;
        let screen_size = camera.logical_target_size()?;
        let viewport_size = viewport_max - viewport_min;
        let adj_cursor_pos =
            cursor_pos_screen - Vec2::new(viewport_min.x, screen_size.y - viewport_max.y);

        let projection = camera.projection_matrix();
        let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
        let near_ndc = projection.project_point3(Vec3::Z).z;
        let cursor_ndc = (adj_cursor_pos / viewport_size) * 2.0 - Vec2::ONE;
        let ndc_to_world: Mat4 = view * projection.inverse();
        let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
        let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
        let ray_direction = far - near;
        Some(Self {
            origin: near.into(),
            direction: ray_direction.into(),
        })
    }

    pub fn intersects_aabb(&self, aabb: &Aabb, model_to_world: &Mat4) -> Option<[f32; 2]> {
        // Transform the ray to model space
        let world_to_model = model_to_world.inverse();
        let ray_dir: Vec3A = world_to_model
            .transform_vector3(self.direction.into())
            .into();
        let ray_origin: Vec3A = world_to_model.transform_point3(self.origin.into()).into();
        // Check if the ray intersects the mesh's AABB. It's useful to work in model space because
        // we can do an AABB intersection test, instead of an OBB intersection test.

        let t_0: Vec3A = (aabb.min() - ray_origin) / ray_dir;
        let t_1: Vec3A = (aabb.max() - ray_origin) / ray_dir;
        let t_min: Vec3A = t_0.min(t_1);
        let t_max: Vec3A = t_0.max(t_1);

        let mut hit_near = t_min.x;
        let mut hit_far = t_max.x;

        if hit_near > t_max.y || t_min.y > hit_far {
            return None;
        }

        if t_min.y > hit_near {
            hit_near = t_min.y;
        }
        if t_max.y < hit_far {
            hit_far = t_max.y;
        }

        if (hit_near > t_max.z) || (t_min.z > hit_far) {
            return None;
        }

        if t_min.z > hit_near {
            hit_near = t_min.z;
        }
        if t_max.z < hit_far {
            hit_far = t_max.z;
        }
        Some([hit_near, hit_far])
    }
}

