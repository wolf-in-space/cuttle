
fn point(input: Point) {
    distance = length(position);
}

fn line(input: Line) {
    let x = abs(position.x) - input.length;
    distance = length(vec2(max(x, 0.0), position.y));
}

fn quad(input: Quad) {
    let d = abs(position) - input.half_size;
    distance = length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
}

fn global_transform_render(input: GlobalTransformRender) {
    position = (input.transform * vec4(position.x, position.y, 0.0, 1.0)).xy;
}

fn rounded(input: Rounded) {
    distance -= input.rounded;
}

fn annular(input: Annular) {
    distance = abs(distance) - input.annular;
}

fn fill_render(input: FillRender) {
    color = input.color;
}

fn distance_gradient(input: DistanceGradient) {
    color = mix(color, input.color, cos(distance * input.interval));
}

fn unioni(input: Unioni) {
    if prev_distance < distance {
        distance = prev_distance;
        color = prev_color;
    }
}

fn subtract(input: Subtract) {
    color = prev_color;
    distance = max(prev_distance, -distance);
}

fn intersect(input: Intersect) {
    if prev_distance > distance {
        distance = prev_distance;
        color = prev_color;
    }
}

fn xor(input: Xor) {
    var inter: f32 = max(prev_distance, distance);
    if prev_distance < distance {
        distance = prev_distance;
        color = prev_color;
    } 
    distance = max(distance, -inter);
}

fn smooth_union(input: SmoothUnion) {
    let mix = clamp(0.5 + 0.5 * (distance - prev_distance) / input.smoothness, 0.0, 1.0);
    let distance_correction = input.smoothness * mix * (1.0 - mix);
    distance = mix(distance, prev_distance, mix) - distance_correction;
    color = mix(color, prev_color, mix);
}

fn smooth_subtract(input: SmoothSubtract) {
    let mix = clamp(0.5 - 0.5 * (distance + prev_distance) / input.smoothness, 0.0, 1.0);
    let distance_correction = input.smoothness * mix * (1.0 - mix);
    distance = mix(prev_distance, -distance, mix) + distance_correction;
    color = prev_color;
}

fn smooth_intersect(input: SmoothIntersect) {
    let mix = clamp(0.5 - 0.5 * (distance - prev_distance) / input.smoothness, 0.0, 1.0);
    let distance_correction = input.smoothness * mix * (1.0 - mix);
    distance = mix(distance, prev_distance, mix) + distance_correction;
    color = mix(color, prev_color, mix);
}

fn smooth_xor(input: SmoothXor) {
    var inter: f32 = max(prev_distance, distance);
    if prev_distance > distance {
        prev_distance = distance;
    } else {
        color = prev_color;
    } 
    distance = inter;
    let mix = clamp(0.5 - 0.5 * (distance + prev_distance) / input.smoothness, 0.0, 1.0);
    let distance_correction = input.smoothness * mix * (1.0 - mix);
    distance = mix(prev_distance, -distance, mix) + distance_correction;
}

fn repetition(input: Repetition) {
    let scale = size / input.repetitions;
    let clamp = input.repetitions - 1.0;
    position -= scale * clamp(round(position / scale), -clamp, clamp);
}

fn morph(input: Morph) {
    distance = mix(prev_distance, distance, input.morph);
    color = mix(prev_color, color, input.morph);
}