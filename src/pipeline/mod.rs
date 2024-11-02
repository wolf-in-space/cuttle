use crate::components::SdfCompCount;
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
        sync_world::MainEntity,
        Render, RenderApp, RenderSet,
    },
    ui::TransparentUi,
};
use draw::DrawSdf;
use queue::{cleanup_batches, prepare_sdfs, queue_sdfs, RenderPhaseBuffers};
use specialization::{
    prepare_view_bind_groups, write_phase_buffers, SdfPipeline, SdfSpecializationData,
};

mod draw;
pub mod extract;
mod queue;
pub mod specialization;

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
pub enum UsePipeline {
    #[default]
    World,
    Ui,
}

#[derive(Debug, Component, PartialEq, Eq, Clone, Hash)]
pub struct SdfPipelineKey {
    pipeline: UsePipeline,
}

pub trait RenderPhase: Send + SortedPhaseItem + CachedRenderPipelinePhaseItem {
    fn phase_item(
        sort: f32,
        entity: (Entity, MainEntity),
        pipeline: CachedRenderPipelineId,
        draw_function: DrawFunctionId,
    ) -> Self;

    fn pipeline() -> UsePipeline;
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

    fn pipeline() -> UsePipeline {
        UsePipeline::World
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

    fn pipeline() -> UsePipeline {
        UsePipeline::Ui
    }
}

pub struct PipelinePlugin;
impl Plugin for PipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            extract::plugin,
            render_phase_plugin::<Transparent2d>,
            render_phase_plugin::<TransparentUi>,
        ));

        app.sub_app_mut(RenderApp)
            .configure_sets(
                Render,
                (
                    Buffer,
                    PrepareBindgroups,
                    ((OpPreparation, Queue), ItemPreperation, WriteBuffers).chain(),
                )
                    .after(RenderSet::ExtractCommands)
                    .before(RenderSet::Render),
            )
            .init_resource::<SpecializedRenderPipelines<SdfPipeline>>()
            .add_event::<SdfSpecializationData>()
            .add_systems(Render, cleanup_batches.in_set(RenderSet::Cleanup));
    }

    fn finish(&self, app: &mut App) {
        let count = app.world().resource::<SdfCompCount>().0;
        let render = app.sub_app_mut(RenderApp);
        let pipeline = SdfPipeline::new(render.world_mut(), count);
        render.insert_resource(pipeline);
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComdfRenderSet {
    Buffer,
    OpPreparation,
    Queue,
    ItemPreperation,
    WriteBuffers,
    PrepareBindgroups,
}
use ComdfRenderSet::*;

fn render_phase_plugin<P: RenderPhase>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<RenderPhaseBuffers<P>>()
        .add_render_command::<P, DrawSdf>()
        .add_systems(
            Render,
            (
                (queue_sdfs::<P>, prepare_sdfs::<P>).chain().in_set(Queue),
                write_phase_buffers::<P>.in_set(WriteBuffers),
                prepare_view_bind_groups::<P>
                    .in_set(PrepareBindgroups)
                    .after(RenderSet::PrepareBindGroups),
            ),
        );
}

/*
fn debug_whatever_the_fuck_is_going_on(
    batch: Single<&SdfBatch>,
    sdfs: Query<(&ExtractedSdf, &ExtractedRenderSdf)>,
    op_buffers: Res<OpBuffers>,
    comp_buffers: Single<EntityRef, With<BufferEntity>>,
    buffer_fns: Res<DebugBufferFns>,
) {
    let ops = op_buffers.ops.get();
    let indices = op_buffers.indices.get();
    println!("DEBUG_THIS_NONSENSE");
    println!("rendering {:?} sdfs", batch.range);

    for (sdf, render) in &sdfs {
        println!(" <<< NEXT >>> ");
        println!("{:?}", sdf);
        println!("{:?}", render);
        for o in render.op_start_index..(render.op_start_index + render.op_count) {
            let op = ops[o as usize];
            assert_eq!(sdf.flag.0, op.flag);
            println!("{:?}", op);
            let indices: Vec<_> = (op.start_index..(op.start_index + op.flag.count_ones()))
                .map(|i| indices[i as usize])
                .collect();
            println!("{:?}", indices);
            for (i, index) in Flag(op.flag).into_iter().zip_eq(indices) {
                (buffer_fns[i])(&comp_buffers, index);
            }
        }
    }
}

 */
