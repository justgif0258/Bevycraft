#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions as fns,
    pbr_bindings,
}
#import bevy_core_pipeline::tonemapping::tone_mapping

struct LayeredVertexInput {
    @builtin(instance_index) instance_index: u32,

    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    #ifdef VERTEX_TANGENTS
        @location(4) tangent: vec4<f32>,
    #endif

    #ifdef VERTEX_COLORS
        @location(7) color: vec4<f32>,
    #endif

    @location(8) layer: u32,
}

struct LayeredVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    #ifdef VERTEX_TANGENTS
        @location(4) world_tangent: vec4<f32>,
    #endif

    #ifdef VERTEX_COLORS
        @location(7) color: vec4<f32>,
    #endif

    @location(8) @interpolate(flat) layer: u32,
}

@vertex
fn vs_main(in: LayeredVertexInput) -> LayeredVertexOutput {
        var out: LayeredVertexOutput;

        var model = mesh_functions::get_world_from_local(in.instance_index);

        out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(in.position, 1.0));

        out.position = mesh_functions::mesh_position_local_to_clip(model, vec4<f32>(in.position, 1.0));

        out.world_normal = mesh_functions::mesh_normal_local_to_world(in.normal, in.instance_index);

        #ifdef VERTEX_TANGENTS
            out.world_tangent = mesh_functions::mesh_tangent_local_to_world(model, in.tangent, in.instance_index);
        #endif

        #ifdef VERTEX_COLORS
            out.color = in.color;
        #endif

        out.uv = in.uv;
        out.layer = in.layer;

        return out;
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var block_texture: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var block_sampler: sampler;

@fragment
fn fs_main(
    @builtin(front_facing) is_front: bool,
    mesh: LayeredVertexOutput,
) -> @location(0) vec4<f32> {
    let layer = mesh.layer;

    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = textureSample(block_texture, block_sampler, mesh.uv, layer);

    #ifdef VERTEX_COLORS
        pbr_input.material.base_color = pbr_input.material.base_color * mesh.color;
    #endif

    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.world_normal = fns::prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front,
    );

    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;

    pbr_input.N = normalize(pbr_input.world_normal);

    #ifdef VERTEX_TANGENTS
        let Nt = textureSampleBias(pbr_bindings::normal_map_texture, pbr_bindings::normal_map_sampler, mesh.uv, view.mip_bias).rgb;
        let TBN = fns::calculate_tbn_mikktspace(mesh.world_normal, mesh.world_tangent);
        pbr_input.N = fns::apply_normal_mapping(
            pbr_input.material.flags,
            TBN,
            double_sided,
            is_front,
            Nt,
        );
    #endif

    pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);

    return tone_mapping(fns::apply_pbr_lighting(pbr_input), view.color_grading);
}