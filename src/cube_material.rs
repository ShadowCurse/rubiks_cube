use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType},
    },
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
#[uniform(0, CubeMaterialUniform)]
pub struct CubeMaterial {
    pub emissive: Color,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub colors: [Color; 7],
}

impl Default for CubeMaterial {
    fn default() -> Self {
        Self {
            emissive: Color::BLACK,
            perceptual_roughness: 0.69,
            metallic: 0.001,
            reflectance: 0.01,
            colors: [
                Color::RED,
                Color::GREEN,
                Color::YELLOW,
                Color::NONE,
                Color::WHITE,
                Color::BLUE,
                Color::rgb(1.0, 0.35, 0.0),
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

#[derive(Clone, Default, ShaderType)]
pub struct CubeMaterialUniform {
    pub emissive: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub alpha_cutoff: f32,
    pub colors: [Vec4; 7],
}

impl AsBindGroupShaderType<CubeMaterialUniform> for CubeMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> CubeMaterialUniform {
        CubeMaterialUniform {
            emissive: self.emissive.into(),
            roughness: self.perceptual_roughness,
            metallic: self.metallic,
            reflectance: self.reflectance,
            alpha_cutoff: 0.5,
            colors: self.colors.map(Into::into),
        }
    }
}
