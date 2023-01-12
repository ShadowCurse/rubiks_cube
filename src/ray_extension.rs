use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb};

pub trait RayExtension {
    fn intersects_aabb(&self, aabb: &Aabb, model_to_world: &Mat4) -> Option<[f32; 2]>;
    fn aabb_plane_normal(&self, t: f32, aabb: &Aabb) -> Vec3;
}

impl RayExtension for Ray {
    fn intersects_aabb(&self, aabb: &Aabb, model_to_world: &Mat4) -> Option<[f32; 2]> {
        // Transform the ray to model space
        let world_to_model = model_to_world.inverse();
        let ray_dir: Vec3A = world_to_model
            .transform_vector3(self.direction)
            .into();
        let ray_origin: Vec3A = world_to_model.transform_point3(self.origin).into();
        // Check if the ray intersects the mesh's AABB. It's useful to work in model space because
        // we can do an AABB intersection test, instead of an OBB intersection test.

        let t_0: Vec3A = (aabb.min() - ray_origin) / ray_dir;
        let t_1: Vec3A = (aabb.max() - ray_origin) / ray_dir;
        let t_min: Vec3A = t_0.min(t_1);
        let t_max: Vec3A = t_0.max(t_1);

        let mut hit_near = t_min.x;
        let mut hit_far = t_max.x;

        if (hit_near > t_max.y) || (t_min.y > hit_far) {
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

    // returns the plane to whith the point of t lies closest
    fn aabb_plane_normal(&self, t: f32, aabb: &Aabb) -> Vec3 {
        let point = self.origin + t * self.direction;

        let mut closest_plane = f32::MAX;
        let mut plane_normal = Vec3::default();
        for (normal, plane_center) in [
            (Vec3::X, aabb.half_extents.x),
            (Vec3::NEG_X, -aabb.half_extents.x),
            (Vec3::Y, aabb.half_extents.y),
            (Vec3::NEG_Y, -aabb.half_extents.y),
            (Vec3::Z, aabb.half_extents.z),
            (Vec3::NEG_Z, -aabb.half_extents.z),
        ] {
            let close_to_plane = (plane_center - point).dot(normal);
            if closest_plane > close_to_plane {
                closest_plane = close_to_plane;
                plane_normal = normal;
            }
        }

        plane_normal
    }
}
