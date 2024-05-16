use crate::components::Border;
use crate::flag::RenderableSdf;
use crate::linefy;
use crate::prelude::{Fill, Gradient};
use crate::render::extract::EntityTranslator;
use crate::render::shader::buffers::SdfStorageBuffer;
use crate::render::shader::lines::Lines;
use crate::render::shader::variants::Calculation::*;
use crate::render::shader::variants::SdfCalculationBuilder;
use crate::scheduling::ComdfRenderSet::*;
use bevy_app::App;
use bevy_comdf_core::prelude::*;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_log::error;
use bevy_render::{Extract, ExtractSchedule, Render, RenderApp};
use bevy_transform::components::GlobalTransform;
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
            GlobalTransform,
            Rotated,
            Bend,
            Elongated,
            Added,
            Annular,
            Fill,
            Gradient,
            Border
        ]
    );

    render_app.add_systems(ExtractSchedule, extract.in_set(Extract));
    render_app.add_systems(
        Render,
        (
            build_flags.in_set(BuildSdfFlags),
            build_shaders.chain().in_set(BuildShadersForComponents),
            prepare_buffers.chain().in_set(BuildBuffersForComponents),
        ),
    );
}

trait RenderSdfComponent: Sized + Component + Clone {
    fn flag() -> RenderableSdf;
    fn flag_system(mut query: Query<&mut RenderableSdf, With<Self>>) {
        query
            .iter_mut()
            .for_each(|mut variant| *variant |= Self::flag());
    }

    fn setup(shader: &mut SdfCalculationBuilder);
    fn setup_system(mut query: Query<&mut SdfCalculationBuilder, With<Self>>) {
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
        query: Extract<Query<(Entity, &Self), With<Sdf>>>,
    ) {
        cmds.insert_or_spawn_batch(
            query
                .into_iter()
                .filter_map(|(e, c)| {
                    translator.0.get(&e).map_or_else(
                        || {
                            error!(
                                "entity '{e:?}' could not be translated during extract of {:?}",
                                Self::flag()
                            );
                            None
                        },
                        |e| Some((*e, c.clone())),
                    )
                })
                .collect_vec(),
        )
    }
}

impl RenderSdfComponent for Point {
    fn flag() -> RenderableSdf {
        RenderableSdf::Point
    }
    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.calc(Distance, stringify!(length(position)))
    }

    fn prep(_render: &mut SdfStorageBuffer, _comp: &Self) {}
}

impl RenderSdfComponent for Line {
    fn flag() -> RenderableSdf {
        RenderableSdf::Line
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("line_length", "f32");
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
    fn flag() -> RenderableSdf {
        RenderableSdf::Rectangle
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("rectangle_size", "vec2<f32>");
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
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Rotated {
    fn flag() -> RenderableSdf {
        RenderableSdf::Rotated
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("rotation", "mat2x2<f32>");
        shader.calc(Position, stringify!(input.rotation * position))
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&Mat2::from_angle(comp.0));
    }
}

impl RenderSdfComponent for Translated {
    fn flag() -> RenderableSdf {
        RenderableSdf::Translated
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("translation", "vec2<f32>");
        shader.calc(Position, stringify!(input.translation - position))
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for GlobalTransform {
    fn flag() -> RenderableSdf {
        RenderableSdf::Transform
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("transform", "mat4x4<f32>");
        shader.calc(
            Position,
            stringify!((input.transform * vec4(position.x, position.y, 0.0, 1.0)).xy),
        )
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.compute_matrix());
    }
}

impl RenderSdfComponent for Added {
    fn flag() -> RenderableSdf {
        RenderableSdf::Added
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("added", "f32");
        shader.calc(Distance, stringify!(result.distance - input.added));
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Annular {
    fn flag() -> RenderableSdf {
        RenderableSdf::Annular
    }
    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("annular_radius", "f32");
        shader.calc(
            Distance,
            stringify!(abs(result.distance) - input.annular_radius),
        );
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Bend {
    fn flag() -> RenderableSdf {
        RenderableSdf::Bend
    }
    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("bend", "f32");
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
        shader.calc(Position, stringify!(bend_point(position, input.bend)))
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Elongated {
    fn flag() -> RenderableSdf {
        RenderableSdf::Stretched
    }
    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("elongated", "vec2<f32>");
        shader.extra(
            Self::flag(),
            linefy!(
                fn elongate_point(point: vec2<f32>, elongate: vec2<f32>) -> vec2<f32> {
                    let q = abs(point) - elongate;
                    return max(q, vec2(0.0)) + min(max(q.x, q.y), 0.0);
                }
            ),
        );
        shader.calc(
            Position,
            stringify!(elongate_point(position, input.elongated)),
        );
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.0)
    }
}

impl RenderSdfComponent for Fill {
    fn flag() -> RenderableSdf {
        RenderableSdf::Fill
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("fill_color", "vec3<f32>");
        shader.calc(PixelColor, stringify!(input.fill_color));
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.0.rgb_to_vec3())
    }
}

impl RenderSdfComponent for Gradient {
    fn flag() -> RenderableSdf {
        RenderableSdf::Gradient
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("gradient_color", "vec3<f32>");
        shader.input("gradient_intervall", "f32");
        shader.calc(
            PixelColor,
            stringify!(mix(
                result.color,
                input.gradient_color,
                cos(result.distance * input.gradient_intervall)
            )),
        );
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.color.rgb_to_vec3());
        render.push(&comp.intervall);
    }
}

impl RenderSdfComponent for Border {
    fn flag() -> RenderableSdf {
        RenderableSdf::Border
    }

    fn setup(shader: &mut SdfCalculationBuilder) {
        shader.input("border_color", "vec3<f32>");
        shader.input("border_thickness", "f32");
        shader.extra(
            Self::flag(),
            linefy!(
                fn add_border(
                    distance: f32,
                    color: vec3<f32>,
                    border_color: vec3<f32>,
                    border_thickness: f32,
                ) -> vec3<f32> {
                    if distance + border_thickness > 0.0 {
                        return border_color;
                    } else {
                        return color;
                    }
                }
            ),
        );
        shader.calc(
            PixelColor,
            stringify!(add_border(
                result.distance,
                result.color,
                input.border_color,
                input.border_thickness,
            )),
        );
    }

    fn prep(render: &mut SdfStorageBuffer, comp: &Self) {
        render.push(&comp.color.rgb_to_vec3());
        render.push(&comp.thickness);
    }
}
