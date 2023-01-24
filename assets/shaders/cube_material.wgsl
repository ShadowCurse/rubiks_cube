#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions

struct CubeMaterial {
    colors: array<vec3<f32>, 7>,
};

@group(1) @binding(0)
var<uniform> material: CubeMaterial;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) normal: vec3<f32>,
};

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(in.position, 1.0));
    out.normal = in.normal;
    return out;
  }

@fragment
fn fragment(
  in: VertexOutput
) -> @location(0) vec4<f32> {
    var normal2 = in.normal + 1.0;
    var index = i32(dot(normal2, vec3<f32>(1.0, 2.0, 3.0)) - 3.0);
    //return vec4<f32>(index, index, index, 1.0);
    return vec4<f32>(material.colors[index], 1.0);
}
