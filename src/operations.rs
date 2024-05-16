use crate::flag::{RenderableSdf, SdfPipelineKey};
use crate::render::extract::EntityTranslator;
use crate::render::shader::buffers::{SdfStorageBuffer, SdfStorageIndex};
use crate::render::shader::lines::*;
use crate::render::shader::loading::{SdfBinding, SdfBindings, SdfShaderRegister};
use crate::render::shader::variants::{Calculation, SdfCalculationBuilder};
use crate::scheduling::ComdfRenderSet::*;
use crate::{linefy, RenderSdf};
use aery::prelude::*;
use bevy_app::prelude::*;
use bevy_comdf_core::prelude::*;
use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_render::{Extract, ExtractSchedule, Render, RenderApp};
use std::marker::PhantomData;
use std::ops::Deref;

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
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

    let (
        build_shaders,
        prepare_buffers,
        build_keys,
        gather_bindings,
        combine_aabbs,
        load_snippets,
        extract,
    ) = system_tuples!(
        [
            build_shader,
            prepare_buffers,
            build_key,
            gather_bindings,
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
                (
                    (add_this_key_entrys, build_keys.chain()).chain(),
                    load_snippets,
                )
                    .in_set(BuildPipelineKeys),
                gather_bindings.in_set(GatherOperationBindings),
                combine_aabbs.after(AfterExtract).before(PrepareBuffers),
                build_shaders.chain().in_set(BuildShadersForOperations),
                prepare_buffers.chain().in_set(BuildBuffersForOperations),
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
        mut operations: Query<(&mut SdfPipelineKey, Relations<Self>)>,
        flags: Query<&RenderableSdf>,
    ) {
        for (mut sdf, edges) in operations.iter_mut() {
            edges.join::<Self>(&flags).for_each(|flag| {
                sdf.0.push((Self::flag(), *flag));
            });
        }
    }

    fn gather_bindings(
        mut operations: Query<(&mut SdfBindings, Relations<Self>)>,
        flags: Query<&SdfBinding>,
    ) {
        for (mut bindings, edges) in operations.iter_mut() {
            edges.join::<Self>(&flags).for_each(|binding| {
                bindings.0.push(binding.0);
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

fn add_this_key_entrys(mut query: Query<(&mut SdfPipelineKey, &RenderableSdf)>) {
    for (mut key, sdf) in query.iter_mut() {
        key.0.push((OperationsFlag::This, *sdf))
    }
}

trait BasicSdfOperation: RenderSdfOperation {
    fn shader(shader: &mut SdfCalculationBuilder, bind: &SdfBinding) {
        let name = Self::flag().func_name();
        let bind = bind.0;
        let input_name = format!("{name}_{bind}_{}", shader.input_len());
        shader.input(input_name.clone(), "u32");
        shader.calc(
            Calculation::Operations,
            format!("sdf_{name}(result, calc_sdf{bind}(input.{input_name}, position))"),
        );
    }

    fn build_shader(
        mut operations: Query<(&mut SdfCalculationBuilder, Relations<Self>)>,
        bindings: Query<&SdfBinding>,
    ) {
        for (mut builder, edges) in operations.iter_mut() {
            builder.operation_snippets.extend(Self::require_snippets());
            edges
                .join::<Self>(&bindings)
                .for_each(|binding| Self::shader(&mut builder, binding));
        }
    }

    fn prepare_buffers(
        mut operations: Query<(&mut SdfStorageBuffer, Relations<Self>)>,
        variants: Query<&SdfStorageIndex>,
    ) {
        for (mut buffer, edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|index| {
                buffer.push(&index.0);
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
    fn shader(shader: &mut SdfCalculationBuilder, SdfBinding(bind): &SdfBinding) {
        let name = Self::flag().func_name();
        let index_name = format!("{name}_{bind}_{}", shader.input_len());
        let value_name = format!("{index_name}_value");
        shader.input(index_name.clone(), "u32");
        shader.input(value_name.clone(), "f32");
        shader.calc(
            Calculation::Operations,
            format!(
                "sdf_{name}(result, calc_sdf{bind}(input.{index_name}, position), input.{value_name})"
            ),
        );
    }

    fn build_shader(
        mut operations: Query<(&mut SdfCalculationBuilder, Relations<Self>)>,
        bindings: Query<&SdfBinding>,
    ) {
        for (mut builder, edges) in operations.iter_mut() {
            builder.operation_snippets.extend(Self::require_snippets());
            edges
                .join::<Self>(&bindings)
                .for_each(|bind| Self::shader(&mut builder, bind));
        }
    }

    fn prepare_buffers(
        mut operations: Query<(
            (&mut SdfStorageBuffer, &OperationsValues<Self>),
            Relations<Self>,
        )>,
        variants: Query<(Entity, &SdfStorageIndex)>,
    ) {
        for ((mut buffer, values), edges) in operations.iter_mut() {
            edges.join::<Self>(&variants).for_each(|(entity, index)| {
                buffer.push(&index.0);
                buffer.push(&values.values.get(&entity).copied().unwrap_or(0.5));
            });
        }
    }

    fn extract(
        mut cmds: Commands,
        translator: Res<EntityTranslator>,
        mut extract: Extract<Query<((Entity, &OperationsValues<Self>), Relations<Self>)>>,
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
    fn shader(shader: &mut SdfCalculationBuilder, SdfBinding(bind): &SdfBinding) {
        shader.input(format!("base_{}", bind), "u32");
        shader.calc(
            Calculation::Operations,
            format!("calc_sdf{b}(input.base_{b}, position)", b = bind),
        )
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
