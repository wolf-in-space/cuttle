use crate::linefy;
use crate::prelude::{FillColor, GradientColor};
use crate::render::shader::buffers::SdfVariantBuffer;
use crate::render::shader::lines::Lines;
use crate::render::shader::variants::Calculation::*;
use crate::render::shader::variants::VariantShaderBuilder;
use crate::scheduling::ComdfRenderPostUpdateSet::*;
use crate::scheduling::ComdfRenderUpdateSet::*;
use crate::{flag::VariantFlag, RenderSdfComponent};
use bevy::app::PostUpdate;
use bevy::app::{App, Update};
use bevy::math::Mat2;
use bevy::prelude::IntoSystemConfigs;
use bevy::render::render_resource::VertexFormat;
use bevy_comdf_core::prelude::*;

pub fn plugin(app: &mut App) {
    let (setup_rendering, prepare_for_frame, build_flags) = system_tuples!(
        [setup_system, prep_system, flag_system],
        [
            Point,
            Line,
            Rectangle,
            Translated,
            Rotated,
            Bend,
            Stretched,
            Added,
            Annular,
            FillColor,
            GradientColor
        ]
    );

    app.add_systems(Update, build_flags.in_set(BuildVariantFlags));
    app.add_systems(
        PostUpdate,
        (
            setup_rendering.chain().in_set(BuildShaders),
            prepare_for_frame.chain().in_set(GatherDataForExtract),
        ),
    );
}

impl RenderSdfComponent for Point {
    fn flag() -> VariantFlag {
        VariantFlag::Point
    }
    fn setup(shader: &mut VariantShaderBuilder) {
        shader.calc(Distance, stringify!(length(position)))
    }

    fn prep(_render: &mut SdfVariantBuffer, _comp: &Self) {}
}

impl RenderSdfComponent for Line {
    fn flag() -> VariantFlag {
        VariantFlag::Line
    }

    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("line_length", VertexFormat::Float32));
        shader.extra(
            Self::flag(),
            linefy!(
                fn dist_to_line(point: vec2<f32>, length: f32) -> f32 {
                    let x = abs(point.x) - length;
                    return length(vec2(max(x, 0.0), point.y));
                }
            ),
        );
        shader.calc(
            Distance,
            stringify!(dist_to_line(position, input.line_length)),
        );
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Rectangle {
    fn flag() -> VariantFlag {
        VariantFlag::Rectangle
    }
    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("rectangle_size", VertexFormat::Float32x2));
        shader.extra(
            Self::flag(),
            linefy!(
                fn dist_to_box(point: vec2<f32>, size: vec2<f32>) -> f32 {
                    let d = abs(point) - size;
                    return length(max(d, vec2(0.0))) + min(max(d.x, d.y), 0.0);
                }
            ),
        );
        shader.calc(
            Distance,
            stringify!(dist_to_box(position, input.rectangle_size)),
        );
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0.to_array())
    }
}

impl RenderSdfComponent for Rotated {
    fn flag() -> VariantFlag {
        VariantFlag::Rotated
    }

    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("rotation_mat_1", VertexFormat::Float32x2));
        shader.input(("rotation_mat_2", VertexFormat::Float32x2));
        shader.calc(
            Position,
            stringify!(mat2x2(input.rotation_mat_1, input.rotation_mat_2) * <prev>),
        )
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        let mat = Mat2::from_angle(comp.0).to_cols_array_2d();
        render.push(&mat[0]);
        render.push(&mat[1]);
    }
}

impl RenderSdfComponent for Translated {
    fn flag() -> VariantFlag {
        VariantFlag::Translated
    }

    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("translation", VertexFormat::Float32x2));
        shader.calc(Position, stringify!(input.translation - <prev>))
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0.to_array())
    }
}

impl RenderSdfComponent for Added {
    fn flag() -> VariantFlag {
        VariantFlag::Added
    }

    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("added", VertexFormat::Float32));
        shader.calc(Distance, stringify!(<prev> - input.added));
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Annular {
    fn flag() -> VariantFlag {
        VariantFlag::Annular
    }
    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("annular_radius", VertexFormat::Float32));
        shader.calc(Distance, stringify!(abs(<prev>) - input.annular_radius));
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Bend {
    fn flag() -> VariantFlag {
        VariantFlag::Bend
    }
    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("bend", VertexFormat::Float32));
        shader.extra(
            Self::flag(),
            linefy!(
                fn bend_point(point: vec2<f32>, bend: f32) -> vec2<f32> {
                    let c = cos(bend * point.x);
                    let s = sin(bend * point.x);
                    let m = mat2x2(c, -s, s, c);
                    return m * point.xy;
                }
            ),
        );
        shader.calc(Position, stringify!(bend_point(<prev>, input.bend)))
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Stretched {
    fn flag() -> VariantFlag {
        VariantFlag::Stretched
    }
    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("stretch", VertexFormat::Float32x2));
        shader.calc(
            Position,
            stringify!(
                <prev> + dot(normalize(<prev>), normalize(input.stretch)) * -input.stretch
            ),
        );
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0.to_array())
    }
}

impl RenderSdfComponent for FillColor {
    fn flag() -> VariantFlag {
        VariantFlag::FillColor
    }

    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("fill_color", VertexFormat::Float32x3));
        shader.calc(PixelColor, stringify!(input.fill_color));
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0.rgb_to_vec3())
    }
}

impl RenderSdfComponent for GradientColor {
    fn flag() -> VariantFlag {
        VariantFlag::GradientColor
    }

    fn setup(shader: &mut VariantShaderBuilder) {
        shader.input(("gradient_color", VertexFormat::Float32x3));
        shader.calc(
            PixelColor,
            stringify!(mix(<prev>, input.gradient_color, abs(result.distance % 50.0))),
        );
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self) {
        render.push(&comp.0.rgb_to_vec3())
    }
}
