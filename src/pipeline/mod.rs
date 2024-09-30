use crate::flag::SdfFlags;
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    math::FloatOrd,
    prelude::*,
    render::{
        render_phase::{
            AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId, PhaseItemExtraIndex,
            SortedPhaseItem,
        },
        render_resource::{CachedRenderPipelineId, SpecializedRenderPipelines},
        Render, RenderApp, RenderSet,
    },
    ui::TransparentUi,
};
use draw::DrawSdf;
use extract::{extract_render_sdf, ExtractedSdfs};
use queue::{prepare_sdfs, queue_sdfs, RenderPhaseBuffers};
use specialization::{
    add_new_sdf_to_pipeline, prepare_view_bind_groups, redo_bindgroups, write_comp_buffers,
    write_phase_buffers, SdfPipeline, SdfSpecializationData,
};

mod draw;
mod extract;
mod queue;
mod specialization;

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
pub enum UsePipeline {
    #[default]
    World,
    Ui,
}

#[derive(Debug, Component, PartialEq, Eq, Clone, Hash)]
pub struct SdfPipelineKey {
    pipeline: UsePipeline,
    flags: SdfFlags,
}

trait RenderPhase: Send + SortedPhaseItem + CachedRenderPipelinePhaseItem {
    fn phase_item(
        sort: f32,
        entity: Entity,
        pipeline: CachedRenderPipelineId,
        draw_function: DrawFunctionId,
    ) -> Self;
}

impl RenderPhase for Transparent2d {
    fn phase_item(
        sort: f32,
        entity: Entity,
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
        entity: Entity,
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
        app.add_plugins((
            render_phase_plugin::<Transparent2d>,
            render_phase_plugin::<TransparentUi>,
        ));

        app.sub_app_mut(RenderApp)
            .configure_sets(
                Render,
                (
                    ProcessEvents,
                    Queue,
                    WriteBuffers,
                    PrepareBindgroups.after(RenderSet::PrepareBindGroups),
                )
                    .chain()
                    .after(RenderSet::ExtractCommands)
                    .before(RenderSet::Render),
            )
            .init_resource::<SpecializedRenderPipelines<SdfPipeline>>()
            .add_event::<SdfSpecializationData>()
            .add_systems(ExtractSchedule, extract_render_sdf)
            .add_systems(
                Render,
                (
                    add_new_sdf_to_pipeline.in_set(ProcessEvents),
                    write_comp_buffers.in_set(WriteBuffers),
                    (redo_bindgroups,).in_set(PrepareBindgroups),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<SdfPipeline>();
        }
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComdfRenderSet {
    ProcessEvents,
    Queue,
    WriteBuffers,
    PrepareBindgroups,
}
use ComdfRenderSet::*;

fn render_phase_plugin<P: RenderPhase>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<ExtractedSdfs<P>>()
        .init_resource::<RenderPhaseBuffers<P>>()
        .add_render_command::<P, DrawSdf>()
        .add_systems(
            Render,
            (
                (queue_sdfs::<P>, prepare_sdfs::<P>).chain().in_set(Queue),
                prepare_view_bind_groups::<P>.in_set(PrepareBindgroups),
                write_phase_buffers::<P>.in_set(WriteBuffers),
            ),
        );
}

/*
TransparentUi {
                sort_key: (FloatOrd(sdf.sort), 0),
                entity,
                pipeline,
                draw_function,
                batch_range: 0..0,
                extra_index: PhaseItemExtraIndex::NONE,
            }
*/
