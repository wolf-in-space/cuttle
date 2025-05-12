#import bevy_render::view::View
#import cuttle::common::VertexOut

struct VertexIn {
    @builtin(vertex_index) index: u32,
    @location(0) translation: vec2<f32>,
    @location(1) bounding_radius: f32,
    @location(2) start: u32,
    @location(3) end: u32,
}

@group(0) @binding(0) 
var<uniform> view: View;

@vertex
fn vertex(input: VertexIn) -> VertexOut {
    let direction = vec2<f32>(f32(input.index & 0x1u) - 0.5, f32((input.index & 0x2u) >> 1u) - 0.5);

    var out: VertexOut;
    out.world_position = direction * input.bounding_radius * 2.0;
    out.world_position += input.translation;
    out.position = view.clip_from_world * vec4(out.world_position, 0.0, 1.0);
    out.start = input.start;
    out.end = input.end;
    out.size = input.bounding_radius;

    return out;
}
