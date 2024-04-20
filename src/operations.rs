use crate::flag::{RenderableSdf, SdfPipelineKey};
use crate::render::extract::EntityTranslator;
use crate::render::shader::buffers::{SdfOperationsBuffer, SdfStorageIndex};
use crate::render::shader::building::SdfShaderBuilder;
use crate::render::shader::lines::*;
use crate::render::shader::loading::SdfShaderRegister;
use crate::scheduling::ComdfRenderSet::*;
use crate::{linefy, RenderSdf};
use aery::prelude::*;
use bevy_app::prelude::*;
use bevy_comdf_core::prelude::*;
use bevy_core::bytes_of;
use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_render::render_resource::VertexFormat;
use bevy_render::{Extract, ExtractSchedule, Render, RenderApp};
use std::marker::PhantomData;
use std::ops::Deref;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        ((
            add_a_if_with_b_and_without_a::<OperationsValues<SmoothUnion>, RenderSdf>,
            add_a_if_with_b_and_without_a::<OperationsValues<SmoothIntersect>, RenderSdf>,
            add_a_if_with_b_and_without_a::<OperationsValues<SmoothSubtract>, RenderSdf>,
            add_a_if_with_b_and_without_a::<OperationsValues<SmoothXOR>, RenderSdf>,
            add_a_if_with_b_and_without_a::<OperationsValues<Merge>, RenderSdf>,
        ),),
    );
    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    let (build_shaders, prepare_buffers, build_keys, combine_aabbs, load_snippets, extract) = system_tuples!(
        [
            build_shader,
            prepare_buffers,
            build_key,
            combine_aabbs,
            load_snippets,
            extract
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

    render_app
        .add_systems(ExtractSchedule, extract.in_set(Extract))
        .add_systems(
            Render,
            (
                // clear_render_keys.before(BuildPipelineKeys),
                (build_keys.chain(), load_snippets).in_set(BuildPipelineKeys),
                (
                    // clear_aabbs,
                    combine_aabbs,
                )
                    .chain()
                    .after(AfterExtract)
                    .before(PrepareBuffers),
                build_shaders.chain().in_set(BuildShaders),
                prepare_buffers.chain().in_set(PrepareBuffers),
            ),
        );
}

type RelationsQuery<'a, 'b, Data, Relation> = Query<'a, 'b, (Data, Relations<Relation>)>;
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
        mut operations: Query<(&mut SdfPipelineKey, Relations<Self>)>,
        variants: Query<&RenderableSdf>,
    ) {
        // println!("build_key_{:?}: {}", Self::flag(), operations.iter().len());
        for (mut sdf, edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|variant| {
                sdf.0.push((Self::flag(), variant.binding));
            });
        }
    }

    fn combine_aabbs(
        mut operations: Query<(&mut AABB, Relations<Self>), With<SdfPipelineKey>>,
        variants: Query<&AABB, Without<SdfPipelineKey>>,
    ) {
        for (mut aabb, edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|variant| {
                *aabb = aabb.combine(variant);
            });
        }
    }
}

trait BasicSdfOperation: RenderSdfOperation {
    fn shader(shader: &mut SdfShaderBuilder, variant: &RenderableSdf) {
        let name = Self::flag().func_name();
        let bind = variant.binding;
        shader.input((format!("{name}_{bind}"), VertexFormat::Uint32));
        shader.operation(format!(
            "sdf_{name}(result, calc_sdf{bind}(input.{name}_{bind}, input.world_position));"
        ));
    }

    fn build_shader(
        mut operations: Query<(&mut SdfShaderBuilder, Relations<Self>)>,
        variants: Query<&RenderableSdf>,
    ) {
        for (mut builder, edges) in operations.iter_mut() {
            builder.operation_snippets.extend(Self::require_snippets());
            edges
                .join::<Self>(&variants)
                .for_each(|variant| Self::shader(&mut builder, variant));
        }
    }

    fn prepare_buffers(
        mut operations: Query<(&mut SdfOperationsBuffer, Relations<Self>)>,
        variants: Query<&SdfStorageIndex>,
    ) {
        for (mut buffer, edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|index| {
                buffer.0.extend_from_slice(bytes_of(&index.0));
            });
        }
    }

    fn extract(
        mut cmds: Commands,
        translator: Res<EntityTranslator>,
        mut extract: Extract<Query<(Entity, Relations<Self>)>>,
        sdfs: Extract<Query<Entity, With<Sdf>>>,
    ) {
        for (target, edges) in extract.iter_mut() {
            let target_render_ent = *translator.0.get(&target).unwrap();
            edges.join::<Self>(sdfs.deref()).for_each(|sdf| {
                let sdf_render_ent = *translator.0.get(&sdf).unwrap();
                cmds.entity(sdf_render_ent).set::<Self>(target_render_ent);
            });
        }
    }
}

pub trait SdfOperationWithValue: RenderSdfOperation {
    fn shader(shader: &mut SdfShaderBuilder, variant: &RenderableSdf) {
        let name = Self::flag().func_name();
        let bind = variant.binding;
        shader.input((format!("{name}_{bind}"), VertexFormat::Uint32));
        shader.input((format!("{name}_{bind}_value"), VertexFormat::Float32));
        shader.operation(format!(
            "sdf_{name}(result, calc_sdf{bind}(input.{name}_{bind}, input.world_position), input.{name}_{bind}_value);"
        ));
    }

    fn build_shader(
        mut operations: Query<(&mut SdfShaderBuilder, Relations<Self>)>,
        variants: Query<&RenderableSdf>,
    ) {
        for (mut builder, edges) in operations.iter_mut() {
            builder.operation_snippets.extend(Self::require_snippets());
            edges
                .join::<Self>(&variants)
                .for_each(|variant| Self::shader(&mut builder, variant));
        }
    }

    fn prepare_buffers(
        mut operations: RelationsQuery<(&mut SdfOperationsBuffer, &OperationsValues<Self>), Self>,
        variants: Query<(Entity, &SdfStorageIndex)>,
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

    fn extract(
        mut cmds: Commands,
        translator: Res<EntityTranslator>,
        mut extract: Extract<RelationsQuery<(Entity, &OperationsValues<Self>), Self>>,
        sdfs: Extract<Query<Entity, With<Sdf>>>,
    ) {
        for ((target, values), edges) in extract.iter_mut() {
            let target_render_ent = *translator.0.get(&target).unwrap();
            cmds.entity(target_render_ent)
                .insert(values.translate_to_render(&translator));
            edges.join::<Self>(sdfs.deref()).for_each(|sdf| {
                let sdf_render_ent = *translator.0.get(&sdf).unwrap();
                cmds.entity(sdf_render_ent).set::<Self>(target_render_ent);
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

impl<Op: SdfOperationWithValue> OperationsValues<Op> {
    fn translate_to_render(&self, translation: &EntityTranslator) -> Self {
        let values = self
            .values
            .iter()
            .map(|(ent, val)| (*translation.0.get(ent).unwrap(), *val))
            .fold(EntityHashMap::default(), |mut accu, (key, val)| {
                accu.insert(key, val);
                accu
            });
        Self {
            values,
            marker: PhantomData,
        }
    }
}

impl<Op: SdfOperationWithValue> Default for OperationsValues<Op> {
    fn default() -> Self {
        Self {
            values: EntityHashMap::default(),
            marker: PhantomData,
        }
    }
}

impl<Op: SdfOperationWithValue> Clone for OperationsValues<Op> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            marker: PhantomData,
        }
    }
}

impl<Op: SdfOperationWithValue> std::fmt::Debug for OperationsValues<Op> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationsValues")
            .field("values", &self.values)
            .finish()
    }
}
/*
fn clear_aabbs(mut query: Query<&mut AABB, With<SdfPipelineKey>>) {
    query
        .iter_mut()
        .for_each(|mut aabb| *aabb = AABB::default())
}
 */
/*
fn clear_render_keys(mut query: Query<&mut SdfPipelineKey>) {
    query.iter_mut().for_each(|mut sdf| sdf.2.clear())
}
 */
impl RenderSdfOperation for Base {
    fn flag() -> OperationsFlag {
        OperationsFlag::Base
    }
}

impl BasicSdfOperation for Base {
    fn shader(shader: &mut SdfShaderBuilder, variant: &RenderableSdf) {
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
