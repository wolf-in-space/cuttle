use std::marker::PhantomData;

use crate::flag::{RenderSdf, RenderableVariant};
use crate::linefy;
use crate::render::shader::buffers::{SdfOperationsBuffer, SdfRenderIndex};
use crate::render::shader::building::SdfShaderBuilder;
use crate::render::shader::lines::*;
use crate::render::shader::loading::SdfShaderRegister;
use crate::scheduling::ComdfRenderPostUpdateSet::*;
use crate::scheduling::ComdfRenderUpdateSet::*;
use aery::prelude::*;
use bevy::core::bytes_of;
use bevy::ecs::entity::EntityHashMap;
use bevy::{prelude::*, render::render_resource::VertexFormat};
use bevy_comdf_core::prelude::*;

pub fn plugin(app: &mut App) {
    let (load_shaders, prepare_for_extract, build_keys, combine_aabbs, load_snippets) = system_tuples!(
        [
            load_shader,
            prepare_for_extract,
            build_key,
            combine_aabbs,
            load_snippets
        ],
        [
            Base,
            Union,
            SmoothUnion,
            Intersect,
            SmoothIntersect,
            Subtract,
            SmoothSubtract,
            XOR,
            SmoothXOR,
            Merge
        ]
    );

    app.add_systems(Startup, load_snippets);
    app.add_systems(
        Update,
        (
            clear_render_keys.before(BuildRenderSdfKeys),
            build_keys.chain().in_set(BuildRenderSdfKeys),
            (
                add_a_if_with_b_and_without_a::<OperationsValues<SmoothUnion>, RenderSdf>,
                add_a_if_with_b_and_without_a::<OperationsValues<SmoothIntersect>, RenderSdf>,
                add_a_if_with_b_and_without_a::<OperationsValues<SmoothSubtract>, RenderSdf>,
                add_a_if_with_b_and_without_a::<OperationsValues<SmoothXOR>, RenderSdf>,
                add_a_if_with_b_and_without_a::<OperationsValues<Merge>, RenderSdf>,
            ),
        ),
    );
    app.add_systems(
        PostUpdate,
        (
            (
                add_a_if_with_b_and_without_a::<AABB, RenderSdf>,
                clear_aabbs,
                combine_aabbs,
            )
                .chain(),
            load_shaders.chain().in_set(BuildShaders),
            prepare_for_extract.chain().in_set(GatherDataForExtract),
        ),
    );
}

pub trait RenderSdfOperation: Relation {
    fn flag() -> OperationsFlag;

    fn snippet() -> Lines {
        Lines::new()
    }

    fn require_snippets() -> Vec<OperationsFlag> {
        vec![Self::flag()]
    }

    fn load_snippets(mut register: ResMut<SdfShaderRegister>) {
        register.snippets.insert(Self::flag(), Self::snippet());
    }

    fn build_key(
        mut operations: Query<(&mut RenderSdf, Relations<Self>)>,
        variants: Query<&RenderableVariant>,
    ) {
        for (mut sdf, edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|variant| {
                sdf.0.push((Self::flag(), variant.binding));
            });
        }
    }

    fn combine_aabbs(
        mut operations: Query<(&mut AABB, Relations<Self>), With<RenderSdf>>,
        variants: Query<&AABB, Without<RenderSdf>>,
    ) {
        for (mut aabb, edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|variant| {
                *aabb = aabb.combine(variant);
            });
        }
    }
}

trait BasicSdfOperation: RenderSdfOperation {
    fn shader(shader: &mut SdfShaderBuilder, variant: &RenderableVariant) {
        let name = Self::flag().func_name();
        let bind = variant.binding;
        shader.input((format!("{name}_{bind}"), VertexFormat::Uint32));
        shader.operation(format!(
            "sdf_{name}(result, calc_sdf{bind}(input.{name}_{bind}, input.world_position));"
        ));
    }

    fn load_shader(
        mut operations: Query<(&mut SdfShaderBuilder, Relations<Self>)>,
        variants: Query<&RenderableVariant>,
    ) {
        for (mut builder, edges) in operations.iter_mut() {
            builder.operation_snippets.extend(Self::require_snippets());
            edges
                .join::<Self>(&variants)
                .for_each(|variant| Self::shader(&mut builder, variant));
        }
    }

    fn prepare_for_extract(
        mut operations: Query<(&mut SdfOperationsBuffer, Relations<Self>)>,
        variants: Query<&SdfRenderIndex>,
    ) {
        for (mut buffer, edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|index| {
                buffer.0.extend_from_slice(bytes_of(&index.0));
            });
        }
    }
}

pub trait SdfOperationWithValue: RenderSdfOperation {
    fn shader(shader: &mut SdfShaderBuilder, variant: &RenderableVariant) {
        let name = Self::flag().func_name();
        let bind = variant.binding;
        shader.input((format!("{name}_{bind}"), VertexFormat::Uint32));
        shader.input((format!("{name}_{bind}_value"), VertexFormat::Float32));
        shader.operation(format!(
            "sdf_{name}(result, calc_sdf{bind}(input.{name}_{bind}, input.world_position), input.{name}_{bind}_value);"
        ));
    }

    fn load_shader(
        mut operations: Query<(&mut SdfShaderBuilder, Relations<Self>)>,
        variants: Query<&RenderableVariant>,
    ) {
        for (mut builder, edges) in operations.iter_mut() {
            builder.operation_snippets.extend(Self::require_snippets());
            edges
                .join::<Self>(&variants)
                .for_each(|variant| Self::shader(&mut builder, variant));
        }
    }

    fn prepare_for_extract(
        mut operations: Query<(
            (&mut SdfOperationsBuffer, &OperationsValues<Self>),
            Relations<Self>,
        )>,
        variants: Query<(Entity, &SdfRenderIndex)>,
    ) {
        for ((mut buffer, values), edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|(entity, index)| {
                buffer.0.extend_from_slice(bytes_of(&index.0));
                buffer.0.extend_from_slice(bytes_of(
                    &values.values.get(&entity).copied().unwrap_or(20.0),
                ));
            });
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Component, Reflect)]
pub enum OperationsFlag {
    This,
    Base,
    Union,
    Intersect,
    Subtract,
    XOR,
    SmoothUnion,
    SmoothIntersect,
    SmoothSubtract,
    SmoothXOR,
    Merge,
}

impl OperationsFlag {
    fn func_name(&self) -> &'static str {
        use OperationsFlag::*;
        match self {
            This => unimplemented!(),
            Base => "base",
            Union => "union",
            Intersect => "intersect",
            Subtract => "subtract",
            XOR => "xor",
            SmoothUnion => "smooth_union",
            SmoothIntersect => "smooth_intersect",
            SmoothSubtract => "smooth_subtract",
            SmoothXOR => "smooth_xor",
            Merge => "merge",
        }
    }
}

#[derive(Component)]
pub struct OperationsValues<Op: SdfOperationWithValue> {
    pub values: EntityHashMap<f32>,
    marker: PhantomData<Op>,
}

impl<Op: SdfOperationWithValue> Default for OperationsValues<Op> {
    fn default() -> Self {
        Self {
            values: EntityHashMap::default(),
            marker: PhantomData,
        }
    }
}

fn clear_aabbs(mut query: Query<&mut AABB, With<RenderSdf>>) {
    query
        .iter_mut()
        .for_each(|mut aabb| *aabb = AABB::default())
}

fn clear_render_keys(mut query: Query<&mut RenderSdf>) {
    query.iter_mut().for_each(|mut sdf| sdf.0.clear())
}

impl RenderSdfOperation for Base {
    fn flag() -> OperationsFlag {
        OperationsFlag::Base
    }
}

impl BasicSdfOperation for Base {
    fn shader(shader: &mut SdfShaderBuilder, variant: &RenderableVariant) {
        shader.input((format!("base_{}", variant.binding), VertexFormat::Uint32));
        shader.operation(linefy!(
            b => variant.binding;
            calc_sdf{b}(input.base_{b}, input.world_position);
        ))
    }
}

impl RenderSdfOperation for Union {
    fn flag() -> OperationsFlag {
        OperationsFlag::Union
    }

    fn snippet() -> Lines {
        linefy! {
            fn sdf_union(r1: SdfResult, r2: SdfResult) -> SdfResult {
                if r1.distance < r2.distance {
                    return r1;
                } else {
                    return r2;
                }
            }
        }
    }
}
impl BasicSdfOperation for Union {}

impl RenderSdfOperation for SmoothUnion {
    fn flag() -> OperationsFlag {
        OperationsFlag::SmoothUnion
    }

    fn snippet() -> Lines {
        linefy! {
            fn sdf_smooth_union(r1: SdfResult, r2: SdfResult, smoothness: f32) -> SdfResult {
                let mix = clamp( 0.5 + 0.5 * (r2.distance - r1.distance) / smoothness, 0.0, 1.0);
                let distance_correction = smoothness * mix * (1.0 - mix);
                return SdfResult(
                    mix( r2.distance, r1.distance, mix ) - distance_correction,
                    mix( r2.color, r1.color, mix ),
                );
            }
        }
    }
}
impl SdfOperationWithValue for SmoothUnion {}

impl RenderSdfOperation for Intersect {
    fn flag() -> OperationsFlag {
        OperationsFlag::Intersect
    }

    fn snippet() -> Lines {
        linefy! {
            fn sdf_intersect(r1: SdfResult, r2: SdfResult) -> SdfResult {
                if r1.distance > r2.distance {
                    return r1;
                } else {
                    return r2;
                }
            }
        }
    }
}
impl BasicSdfOperation for Intersect {}

impl RenderSdfOperation for SmoothIntersect {
    fn flag() -> OperationsFlag {
        OperationsFlag::SmoothIntersect
    }

    fn snippet() -> Lines {
        linefy! {
            fn sdf_smooth_intersect(r1: SdfResult, r2: SdfResult, smoothness: f32) -> SdfResult {
                let mix = clamp( 0.5 - 0.5 * (r2.distance - r1.distance) / smoothness, 0.0, 1.0);
                let distance_correction = smoothness * mix * (1.0 - mix);
                return SdfResult(
                    mix( r2.distance, r1.distance, mix ) - distance_correction,
                    mix( r2.color, r1.color, mix ),
                );
            }
        }
    }
}
impl SdfOperationWithValue for SmoothIntersect {}

impl RenderSdfOperation for Subtract {
    fn flag() -> OperationsFlag {
        OperationsFlag::Subtract
    }

    fn snippet() -> Lines {
        linefy! {
            fn sdf_subtract(r1: SdfResult, r2: SdfResult) -> SdfResult {
                if r1.distance > -r2.distance {
                    return r1;
                } else {
                    return SdfResult(-r2.distance, r1.color);
                }
            }
        }
    }
}
impl BasicSdfOperation for Subtract {}

impl RenderSdfOperation for SmoothSubtract {
    fn flag() -> OperationsFlag {
        OperationsFlag::SmoothSubtract
    }
    fn snippet() -> Lines {
        linefy! {
            fn sdf_smooth_subtract(r1: SdfResult, r2: SdfResult, smoothness: f32) -> SdfResult {
                let mix = clamp( 0.5 - 0.5 * ( r1.distance + r2.distance ) / smoothness, 0.0, 1.0 );
                let distance_correction = smoothness * mix * (1.0 - mix);
                return SdfResult(
                    mix( r1.distance, -r2.distance, mix ) + distance_correction,
                    r1.color,
                );
            }
        }
    }
}
impl SdfOperationWithValue for SmoothSubtract {}

impl RenderSdfOperation for XOR {
    fn flag() -> OperationsFlag {
        OperationsFlag::XOR
    }

    fn require_snippets() -> Vec<OperationsFlag> {
        vec![
            OperationsFlag::XOR,
            OperationsFlag::Union,
            OperationsFlag::Intersect,
            OperationsFlag::Subtract,
        ]
    }

    fn snippet() -> Lines {
        linefy! {
            fn sdf_xor(r1: SdfResult, r2: SdfResult) -> SdfResult {
                return sdf_subtract(sdf_union(r1, r2), sdf_intersect(r1, r2));
            }
        }
    }
}
impl BasicSdfOperation for XOR {}

impl RenderSdfOperation for SmoothXOR {
    fn flag() -> OperationsFlag {
        OperationsFlag::SmoothXOR
    }

    fn require_snippets() -> Vec<OperationsFlag> {
        vec![
            OperationsFlag::SmoothXOR,
            OperationsFlag::Union,
            OperationsFlag::Intersect,
            OperationsFlag::SmoothSubtract,
        ]
    }

    fn snippet() -> Lines {
        linefy! {
            fn sdf_smooth_xor(r1: SdfResult, r2: SdfResult, smoothness: f32) -> SdfResult {
                return sdf_smooth_subtract(sdf_union(r1, r2), sdf_intersect(r1, r2), smoothness);
            }
        }
    }
}
impl SdfOperationWithValue for SmoothXOR {}

impl RenderSdfOperation for Merge {
    fn flag() -> OperationsFlag {
        OperationsFlag::Merge
    }

    fn snippet() -> Lines {
        linefy! {
            fn sdf_merge(r1: SdfResult, r2: SdfResult, value: f32) -> SdfResult {
                return SdfResult(mix(r1.distance, r2.distance, value), mix(r1.color, r2.color, value));
            }
        }
    }
}
impl SdfOperationWithValue for Merge {}
