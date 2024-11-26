#import bevy_render::view::View
#import cuttle::common::VertexOut

struct VertexIn {
    @builtin(vertex_index) index: u32,
    @location(0) translation: vec2<f32>,
    @location(1) bounding_radius: f32,
    @location(2) start_index: u32,
    @location(3) op_count: u32,
}

@group(0) @binding(0) 
var<uniform> view: View;

@vertex
fn vertex(input: VertexIn) -> VertexOut {
    let vertex_x = f32(input.index & 0x1u) - 0.5;
    let vertex_y = f32((input.index & 0x2u) >> 1u) - 0.5;
    let vertex_direction = vec2<f32>(vertex_x, vertex_y);

    var out: VertexOut;
    out.world_position = vertex_direction * input.bounding_radius * 4.0;
    out.world_position += input.translation;
    out.position = view.clip_from_world * vec4(out.world_position, 0.0, 1.0);
    out.start_index = input.start_index;
    out.op_count = input.op_count;
    out.size = input.bounding_radius;

    return out;
}