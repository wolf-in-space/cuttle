#import bevy_comdf::gen
#import bevy_comdf::common::VertexOut

struct Op {
	start_index: u32,
	flag: u32,
}

@group(1) @binding(0) var<storage, read> operations: array<Op>;
@group(1) @binding(1) var<storage, read> indices: array<u32>;

@fragment
fn fragment(vert: VertexOut) -> @location(0) vec4<f32> {
	gen::size = vert.size;
    for (var i: u32 = vert.start_index; i < vert.start_index + vert.op_count; i++) {
		gen::position = vert.world_position;
		gen::prev_distance = gen::distance;
		gen::prev_color = gen::color;

		let op = operations[i];
		operation(op);
	}

    let alpha = step(0.0, -gen::distance);
    return vec4(gen::color, alpha);
}

fn operation(op: Op) {
	var flag = op.flag;
	var index = op.start_index;
	while flag > 0 {
		let comp_id = firstTrailingBit(flag);
		flag = flag & (flag - 1);
		gen::component(comp_id, indices[index]);
		index += u32(1);
	}
}