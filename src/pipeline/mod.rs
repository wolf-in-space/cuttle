use bevy::render::render_phase::SortedPhaseItem;
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    math::FloatOrd,
    prelude::*,
    render::{
        render_phase::{
            AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId, PhaseItemExtraIndex,
        },
        render_resource::{CachedRenderPipelineId, SpecializedRenderPipelines},
        sync_world::MainEntity,
        Render, RenderApp, RenderSet,
    },
    ui::TransparentUi,
};
use draw::DrawSdf;
use queue::{
    cleanup_batches, cuttle_prepare_sorted_for_group, cuttle_queue_sorted_for_group, GroupInstanceBuffer,
};
use specialization::{
    prepare_view_bind_groups, write_group_buffer, CuttlePipeline,
};
use std::any::TypeId;

mod draw;
pub mod extract;
mod queue;
pub mod specialization;

#[derive(Debug, Component, PartialEq, Eq, Clone, Hash)]
pub struct CuttlePipelineKey {
    group_id: TypeId,
    multisample_count: u32,
    has_depth: bool,
}

pub trait SortedCuttlePhaseItem: Send + CachedRenderPipelinePhaseItem + SortedPhaseItem {
    fn phase_item(
        sort: f32,
        entity: (Entity, MainEntity),
        pipeline: CachedRenderPipelineId,
        draw_function: DrawFunctionId,
    ) -> Self;
    fn multisample_count() -> u32;
    fn depth() -> bool;
}

impl SortedCuttlePhaseItem for Transparent2d {
    fn phase_item(
        sort: f32,
        entity: (Entity, MainEntity),
        pipeline: CachedRenderPipelineId,
        draw_function: DrawFunctionId,
    ) -> Self {
        Transparent2d {
            sort_key: FloatOrd(sort),
            entity,
            pipeline,
            draw_function,
            batch_range: 0..0,
            extra_index: PhaseItemExtraIndex::NONE,
        }
    }

    fn multisample_count() -> u32 {
        4
    }

    fn depth() -> bool {
        true
    }
}

impl SortedCuttlePhaseItem for TransparentUi {
    fn phase_item(
        sort: f32,
        entity: (Entity, MainEntity),
        pipeline: CachedRenderPipelineId,
        draw_function: DrawFunctionId,
    ) -> Self {
        TransparentUi {
            sort_key: (FloatOrd(sort + 0.268473), entity.0.index()),
            entity,
            pipeline,
            draw_function,
            batch_range: 0..0,
            extra_index: PhaseItemExtraIndex::NONE,
        }
    }

    fn multisample_count() -> u32 {
        1
    }

    fn depth() -> bool {
        false
    }
}

pub struct PipelinePlugin;
impl Plugin for PipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(extract::plugin);

        app.sub_app_mut(RenderApp)
            .configure_sets(
                Render,
                (
                    ComponentBuffers,
                    PrepareBindGroups.after(RenderSet::PrepareBindGroups),
                    PrepareIndices.before(ItemPreparation),
                    PrepareBounds.before(ItemPreparation),
                    (Queue, ItemPreparation, WriteBuffers).chain(),
                )
                    .after(RenderSet::ExtractCommands)
                    .before(RenderSet::Render),
            )
            .init_resource::<SpecializedRenderPipelines<CuttlePipeline>>()
            .add_systems(
                Render,
                (
                    cleanup_batches.in_set(RenderSet::Cleanup),
                    prepare_view_bind_groups
                        .in_set(PrepareBindGroups),
                ),
            );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CuttleRenderSet {
    ComponentBuffers,
    PrepareIndices,
    PrepareBounds,
    Queue,
    ItemPreparation,
    WriteBuffers,
    PrepareBindGroups,
}
use crate::groups::CuttleGroup;
use CuttleRenderSet::*;

pub(crate) fn render_group_plugin<G: CuttleGroup>(app: &mut App) {
    app.init_resource::<GroupInstanceBuffer<G>>()
        .add_render_command::<G::Phase, DrawSdf<G>>()
        .add_systems(
            Render,
            (
                cuttle_queue_sorted_for_group::<G>.in_set(Queue),
                cuttle_prepare_sorted_for_group::<G>.in_set(ItemPreparation),
                write_group_buffer::<G>.in_set(WriteBuffers),
            ),
        );
}
