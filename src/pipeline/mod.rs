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
use queue::{cleanup_batches, prepare_sdfs, queue_sdfs, RenderPhaseBuffers};
use specialization::{
    prepare_view_bind_groups, write_phase_buffers, CuttlePipeline, CuttleSpecializationData,
};

mod draw;
pub mod extract;
mod queue;
pub mod specialization;

#[derive(Debug, Component, PartialEq, Eq, Clone, Hash)]
pub struct SdfPipelineKey {
    group_id: GroupId,
}

pub trait RenderPhase: Send + CachedRenderPipelinePhaseItem {
    fn phase_item(
        sort: f32,
        entity: (Entity, MainEntity),
        pipeline: CachedRenderPipelineId,
        draw_function: DrawFunctionId,
    ) -> Self;
}

impl RenderPhase for Transparent2d {
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
}

impl RenderPhase for TransparentUi {
    fn phase_item(
        sort: f32,
        entity: (Entity, MainEntity),
        pipeline: CachedRenderPipelineId,
        draw_function: DrawFunctionId,
    ) -> Self {
        TransparentUi {
            sort_key: (FloatOrd(sort), 0),
            entity,
            pipeline,
            draw_function,
            batch_range: 0..0,
            extra_index: PhaseItemExtraIndex::NONE,
        }
    }
}

pub struct PipelinePlugin;
impl Plugin for PipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((extract::plugin, render_phase_plugin::<Transparent2d>));

        app.sub_app_mut(RenderApp)
            .configure_sets(
                Render,
                (
                    Buffer,
                    PrepareBindgroups,
                    (OpPreparation, Queue, ItemPreparation, WriteBuffers).chain(),
                )
                    .after(RenderSet::ExtractCommands)
                    .before(RenderSet::Render),
            )
            .init_resource::<SpecializedRenderPipelines<CuttlePipeline>>()
            .add_event::<CuttleSpecializationData>()
            .add_systems(Render, cleanup_batches.in_set(RenderSet::Cleanup));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CuttleRenderSet {
    Buffer,
    OpPreparation,
    Queue,
    ItemPreparation,
    WriteBuffers,
    PrepareBindgroups,
}
use crate::groups::GroupId;
use CuttleRenderSet::*;

fn render_phase_plugin<P: RenderPhase>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<RenderPhaseBuffers>()
        .add_render_command::<P, DrawSdf>()
        .add_systems(
            Render,
            (
                queue_sdfs.in_set(Queue),
                prepare_sdfs.in_set(ItemPreparation),
                write_phase_buffers.in_set(WriteBuffers),
                prepare_view_bind_groups
                    .in_set(PrepareBindgroups)
                    .after(RenderSet::PrepareBindGroups),
            ),
        );
}
