
fn sdf() {
    color.w *= step(0.0, -distance);
    // color.w *= smoothstep(0.0, 1.0, -distance);
}

fn prepare_base() {
    position = vertex.world_position;
}

fn circle(radius: f32) {
    distance = length(position) - radius;
}

fn line(length: f32) {
    let x = abs(position.x) - length;
    distance = length(vec2(max(x, 0.0), position.y));
}

fn quad(half_size: vec2<f32>) {
    let d = abs(position) - half_size;
    distance = length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
}

fn global_transform(transform: mat4x4<f32>) {
    position = (transform * vec4(position.x, position.y, 0.0, 1.0)).xy;
}

fn rounded(rounded: f32) {
    distance -= rounded;
}

fn annular(annular: f32) {
    distance = abs(distance) - annular;
}

fn fill(fill_color: vec4<f32>) {
    color = fill_color;
}

fn distance_gradient(input: DistanceGradient) {
    color = mix(color, input.color, cos(distance * input.interval));
}

fn force_field_alpha() {
    color.w = smoothstep(0.0, -distance, 1.0);
}

fn prepare_operation() {
    prev_distance = distance;
    prev_color = color;
}

fn unioni() {
    if prev_distance < distance {
        distance = prev_distance;
        color = prev_color;
    }
}

fn subtract() {
    color = prev_color;
    distance = max(prev_distance, -distance);
}

fn intersect() {
    if prev_distance > distance {
        distance = prev_distance;
        color = prev_color;
    }
}

fn xor() {
    var inter: f32 = max(prev_distance, distance);
    if prev_distance < distance {
        distance = prev_distance;
        color = prev_color;
    } 
    distance = max(distance, -inter);
}

fn smooth_union(smoothness: f32) {
    let mix = clamp(0.5 + 0.5 * (distance - prev_distance) / smoothness, 0.0, 1.0);
    let distance_correction = smoothness * mix * (1.0 - mix);
    distance = mix(distance, prev_distance, mix) - distance_correction;
    color = mix(color, prev_color, mix);
}

fn smooth_subtract(smoothness: f32) {
    let mix = clamp(0.5 - 0.5 * (distance + prev_distance) / smoothness, 0.0, 1.0);
    let distance_correction = smoothness * mix * (1.0 - mix);
    distance = mix(prev_distance, -distance, mix) + distance_correction;
    color = prev_color;
}

fn smooth_intersect(smoothness: f32) {
    let mix = clamp(0.5 - 0.5 * (distance - prev_distance) / smoothness, 0.0, 1.0);
    let distance_correction = smoothness * mix * (1.0 - mix);
    distance = mix(distance, prev_distance, mix) + distance_correction;
    color = mix(color, prev_color, mix);
}

fn smooth_xor(smoothness: f32) {
    var inter: f32 = max(prev_distance, distance);
    if prev_distance > distance {
        prev_distance = distance;
    } else {
        color = prev_color;
    } 
    distance = inter;
    let mix = clamp(0.5 - 0.5 * (distance + prev_distance) / smoothness, 0.0, 1.0);
    let distance_correction = smoothness * mix * (1.0 - mix);
    distance = mix(prev_distance, -distance, mix) + distance_correction;
}

fn repetition(input: Repetition) {
    let scale = vertex.size / input.repetitions;
    let clamp = input.repetitions - 1.0;
    position -= scale * clamp(round(position / scale), -clamp, clamp);
}

fn morph(morph: f32) {
    distance = mix(prev_distance, distance, morph);
    color = mix(prev_color, color, morph);
}

fn stretch(stretch: vec2<f32>) {
    position /= dot(normalize(position), normalize(stretch)) * length(stretch);
}
