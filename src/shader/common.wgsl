#define_import_path bevy_comdf::common

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec2<f32>,
    @location(1) start_index: u32,
    @location(2) op_count: u32,
    @location(3) size: f32,
}