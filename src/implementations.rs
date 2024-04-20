use crate::flag::{RenderableSdf, VariantFlag};
use crate::linefy;
use crate::prelude::{FillColor, GradientColor};
use crate::render::extract::EntityTranslator;
use crate::render::shader::buffers::SdfStorageBuffer;
use crate::render::shader::lines::Lines;
use crate::render::shader::variants::Calculation::*;
use crate::render::shader::variants::VariantShaderBuilder;
use crate::scheduling::ComdfRenderSet::*;
use bevy_app::App;
use bevy_comdf_core::prelude::*;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_render::render_resource::VertexFormat;
use bevy_render::{Extract, ExtractSchedule, Render, RenderApp};
use glam::Mat2;
use itertools::Itertools;

pub fn plugin(app: &mut App) {
    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    let (build_shaders, prepare_buffers, build_flags, extract) = system_tuples!(
        [setup_system, prep_system, flag_system, extract],
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

    render_app.add_systems(ExtractSchedule, extract.in_set(Extract));
    render_app.add_systems(
        Render,
        (
            build_flags.in_set(BuildSdfFlags),
            build_shaders.chain().in_set(BuildShaders),
            prepare_buffers.chain().in_set(BuildBuffers),
        ),
    );
}

trait RenderSdfComponent: Sized + Component + Clone {
    fn flag() -> VariantFlag;
    fn flag_system(mut query: Query<&mut RenderableSdf, With<Self>>) {
        query
            .iter_mut()
            .for_each(|mut variant| variant.flag |= Self::flag());
    }

    fn setup(shader: &mut VariantShaderBuilder);
    fn setup_system(mut query: Query<&mut VariantShaderBuilder, With<Self>>) {
        query.iter_mut().for_each(|mut comp| Self::setup(&mut comp));
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self);
    fn prep_system(mut query: Query<(&mut SdfStorageBuffer, &Self)>) {
        query.iter_mut().for_each(|(mut buffer, comp)| {
            Self::prep(&mut buffer, comp);
        });
    }

    fn extract(
        mut cmds: Commands,
        translator: Res<EntityTranslator>,
        query: Extract<Query<(Entity, &Self)>>,
    ) {
        cmds.insert_or_spawn_batch(
            query
                .into_iter()
                .map(|(e, c)| (*translator.0.get(&e).unwrap(), c.clone()))
                .collect_vec(),
        )
    }
}

impl RenderSdfComponent for Point {
    fn flag() -> VariantFlag {
        VariantFlag::Point
    }
    fn setup(shader: &mut VariantShaderBuilder) {
        shader.calc(Distance, stringify!(length(position)))
    }

    fn prep(_render: &mut SdfStorageBuffer, _comp: &Self) {}
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
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

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.0.rgb_to_vec3())
    }
}
