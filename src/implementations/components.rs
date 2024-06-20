use super::calculations::{Distance, PixelColor, Position};
use crate::components::buffer::BufferType;
use crate::components::buffer::SdfBuffer;
use crate::components::colors::Border;
use crate::components::{RegisterSdfRenderCompAppExt, RenderSdfComponent};
use crate::linefy;
use crate::prelude::{Fill, Gradient};
use crate::shader::calculations::SdfCalculation;
use crate::shader::lines::Lines;
use crate::shader::CompShaderInfo;
use bevy::app::App;
use bevy::color::{ColorToComponents, LinearRgba};
use bevy::math::*;
use bevy::prelude::default;
use bevy::transform::components::GlobalTransform;
use bevy_comdf_core::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_sdf_render_comp::<Point>()
        .register_sdf_render_comp::<Line>()
        .register_sdf_render_comp::<Rectangle>()
        .register_sdf_render_comp::<Rotated>()
        .register_sdf_render_comp::<Translated>()
        .register_sdf_render_comp::<GlobalTransform>()
        .register_sdf_render_comp::<Added>()
        .register_sdf_render_comp::<Annular>()
        .register_sdf_render_comp::<Bend>()
        .register_sdf_render_comp::<Elongated>()
        .register_sdf_render_comp::<Fill>()
        .register_sdf_render_comp::<Gradient>()
        .register_sdf_render_comp::<Border>();
}

impl RenderSdfComponent for Point {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            calculations: vec![Distance::calc(1000, "length(position)")],
            ..default()
        }
    }

    fn push_to_buffer(&self, _: &mut SdfBuffer) {}
}

impl RenderSdfComponent for Line {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![f32::shader_input("line_length")],
            calculations: vec![Distance::calc(
                1000,
                "dist_to_line(position, input.line_length)",
            )],
            snippets: linefy!(
                fn dist_to_line(point: vec2<f32>, length: f32) -> f32 {
                    let x = abs(point.x) - length;
                    return length(vec2(max(x, 0.0), point.y));
                }
            ),
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&self.0)
    }
}

impl RenderSdfComponent for Rectangle {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![Vec2::shader_input("rectangle_size")],
            calculations: vec![Distance::calc(
                1000,
                "dist_to_box(position, input.rectangle_size)",
            )],
            snippets: linefy!(
                fn dist_to_box(point: vec2<f32>, size: vec2<f32>) -> f32 {
                    let d = abs(point) - size;
                    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
                }
            ),
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&self.0)
    }
}

impl RenderSdfComponent for Rotated {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![Mat2::shader_input("rotation")],
            calculations: vec![Position::calc(2000, "input.rotation * position")],
            ..default()
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&Mat2::from_angle(self.0));
    }
}

impl RenderSdfComponent for Translated {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![Vec2::shader_input("translation")],
            calculations: vec![Position::calc(4000, "input.translation - position")],
            ..default()
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&self.0)
    }
}

impl RenderSdfComponent for GlobalTransform {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![Mat4::shader_input("transform")],
            calculations: vec![Position::calc(
                3000,
                "(input.transform * vec4(position.x, position.y, 0.0, 1.0)).xy",
            )],
            ..default()
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&self.compute_matrix().to_cols_array());
    }
}

impl RenderSdfComponent for Added {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![f32::shader_input("added")],
            calculations: vec![Distance::calc(2000, "result.distance - input.added")],
            ..default()
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&self.0)
    }
}

impl RenderSdfComponent for Annular {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![f32::shader_input("annular")],
            calculations: vec![Distance::calc(3000, "abs(result.distance) - input.annular")],
            ..default()
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&self.0)
    }
}

impl RenderSdfComponent for Bend {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![f32::shader_input("bend")],
            calculations: vec![Position::calc(6000, "bend_point(position, input.bend)")],
            snippets: linefy!(
                fn bend_point(point: vec2<f32>, bend: f32) -> vec2<f32> {
                    let c = cos(bend * point.x);
                    let s = sin(bend * point.x);
                    let m = mat2x2(c, -s, s, c);
                    return m * point.xy;
                }
            ),
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&self.0)
    }
}

impl RenderSdfComponent for Elongated {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![Vec2::shader_input("elongated")],
            calculations: vec![Position::calc(
                7000,
                "elongate_point(position, input.elongated",
            )],
            snippets: linefy!(
                fn elongate_point(point: vec2<f32>, elongate: vec2<f32>) -> vec2<f32> {
                    let q = abs(point) - elongate;
                    return max(q, vec2(0.0)) + min(max(q.x, q.y), 0.0);
                }
            ),
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&self.0)
    }
}

impl RenderSdfComponent for Fill {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![Vec3::shader_input("fill")],
            calculations: vec![PixelColor::calc(2000, "input.fill")],
            ..default()
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        let value = LinearRgba::to_vec3(self.0.into());
        // println!(
        //     "Fill buffer: pos={}, value={} stride={}",
        //     buffer.current_index, value, buffer.stride
        // );
        buffer.push(&value);
        // println!("Buff = {:?}", buffer.buffer.values());
    }
}

impl RenderSdfComponent for Gradient {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![
                Vec3::shader_input("gradient_color"),
                f32::shader_input("gradient_intervall"),
            ],
            calculations: vec![PixelColor::calc(3000,
                "mix(result.color, input.gradient_color, cos(result.distance * input.gradient_intervall))",
            )],
            ..default()
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&LinearRgba::to_vec3(self.color.into()));
        buffer.push(&self.intervall);
    }
}

impl RenderSdfComponent for Border {
    fn shader_info() -> CompShaderInfo {
        CompShaderInfo {
            inputs: vec![
                Vec3::shader_input("border_color"),
                f32::shader_input("border_thickness"),
            ],
            calculations: vec![PixelColor::calc(4000,
                "add_border(result.distance, result.color, input.border_color, input.border_thickness)",
            )],
            snippets: linefy!(
                fn add_border(distance: f32, color: vec3<f32>, border_color: vec3<f32>, border_thickness: f32) -> vec3<f32> {
                    if distance + border_thickness > 0.0 {
                        return border_color;
                    } else {
                        return color;
                    }
                }
            )
        }
    }

    fn push_to_buffer(&self, buffer: &mut SdfBuffer) {
        buffer.push(&LinearRgba::from(self.color).to_vec3());
        buffer.push(&self.thickness);
    }
}
