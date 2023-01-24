use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CubeMaterial {
    #[uniform(0)]
    pub colors: [Color; 7],
}

impl Default for CubeMaterial {
    fn default() -> Self {
        Self {
            colors: [
                Color::RED,
                Color::GREEN,
                Color::YELLOW,
                Color::NONE,
                Color::WHITE,
                Color::BLUE,
                Color::ORANGE,
            ],
        }
    }
}

impl Material for CubeMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/cube_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/cube_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}
