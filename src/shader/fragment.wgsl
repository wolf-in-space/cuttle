#import cuttle::common::VertexOut

@group(1) @binding(0) var<storage, read> indices: array<u32>;

@fragment
fn fragment(vert: VertexOut) -> @location(0) vec4<f32> {
    vertex = vert;
    for (var i: u32 = vert.start; i < vert.end; i++) {
        let combined = indices[i];
        let pos = combined & 255;
        let index = combined >> 8;
		component(pos, index);
	}
	
    color.w = 1.0;
    color.w *= smoothstep(0.0, 1.0, -distance);
    return color;
}