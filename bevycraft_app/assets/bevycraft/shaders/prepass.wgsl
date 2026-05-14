#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,

    @location(8) layer: u32,
}

@vertex
fn vertex(in: Vertex) -> @builtin(position) vec4<f32> {
    let world_from_local = get_world_from_local(in.instance_index);

    return mesh_position_local_to_clip(world_from_local, vec4<f32>(in.position, 1.0));
}