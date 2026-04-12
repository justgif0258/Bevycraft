#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_functions,
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions as fns,
    pbr_bindings,
}
#import bevy_core_pipeline::tonemapping::tone_mapping

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var my_array_texture: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var my_array_texture_sampler: sampler;

@builtin
fn fragment(
    @builtin(front_facing) is_front: bool,
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let layer = mesh_functions::get_tag(mesh.instance_index);

    var pbr_input: PbrInput = pbr_input_new();
}
