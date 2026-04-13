@group(0) @binding(0) var block_texture: texture_2d_array<f32>;
@group(0) @binding(1) var block_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv:       vec2<f32>,
    @location(2) layer:    u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       uv:            vec2<f32>,
    @location(1)       layer:         u32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = /* camera matrix */ * vec4(in.position, 1.0);
    out.uv    = in.uv;
    out.layer = in.layer;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(block_texture, block_sampler, in.uv, in.layer);
}