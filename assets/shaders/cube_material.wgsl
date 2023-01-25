#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct CubeMaterial {
    emissive: vec4<f32>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    colors: array<vec4<f32>, 7>,
};

@group(1) @binding(0)
var<uniform> material: CubeMaterial;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
    @location(4) position: vec3<f32>,
    @location(5) normal: vec3<f32>,
    @location(6) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var model = mesh.model;

    out.world_normal = mesh_normal_local_to_world(vertex.normal);
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.clip_position = mesh_position_world_to_clip(out.world_position);
    out.position = vertex.position;
    out.normal = vertex.normal;
    out.uv = vertex.uv;

    return out;
}

struct FragmentInput {
    @builtin(position) frag_coord: vec4<f32>,
    @builtin(front_facing) is_front: bool,
    #import bevy_pbr::mesh_vertex_output
    @location(4) position: vec3<f32>,
    @location(5) normal: vec3<f32>,
    @location(6) uv: vec2<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var normal2 = in.normal + 1.0;
    var index = i32(dot(normal2, vec3<f32>(1.0, 2.0, 3.0)) - 3.0);
    var output_color = material.colors[index];
    if ((in.position.x * in.position.x + in.position.y * in.position.y + in.position.z * in.position.z) > 0.006) {
      output_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let flags: u32 = 0u;
    var pbr_input: PbrInput;

    pbr_input.material.base_color = output_color;
    pbr_input.material.reflectance = material.reflectance;
    pbr_input.material.flags = flags;
    pbr_input.material.alpha_cutoff = 0.5;

    var emissive: vec4<f32> = material.emissive;
    pbr_input.material.emissive =  emissive;

    var metallic: f32 = material.metallic;
    var perceptual_roughness: f32 = material.perceptual_roughness;
    pbr_input.material.metallic = metallic;
    pbr_input.material.perceptual_roughness = perceptual_roughness;

    var occlusion: f32 = 0.5;
    pbr_input.occlusion = occlusion;

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        false,
        in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
        flags,
        pbr_input.world_normal,
        in.uv,
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
    output_color = pbr(pbr_input);

    return output_color;
}
