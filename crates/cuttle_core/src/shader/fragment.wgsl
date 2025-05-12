#import cuttle::common::VertexOut

@group(1) @binding(0) var<storage, read> indices: array<u32>;

var<private> vertex: VertexOut;
var<private> color: vec4<f32>;

@fragment
fn fragment(vert: VertexOut) -> @location(0) vec4<f32> {
    vertex = vert;

    for (var i: u32 = vert.start; i < vert.end; i++) {
        let combined = indices[i];
        let pos = combined & 255;
        let index = combined >> 8;
        component(pos, index);
    }

    return color;
}
