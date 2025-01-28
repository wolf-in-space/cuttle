#define_import_path cuttle::common

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec2<f32>,
    @location(1) start: u32,
    @location(2) end: u32,
    @location(3) size: f32,
}