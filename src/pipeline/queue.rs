use super::{
    draw::DrawSdf,
    extract::{ExtractedRenderSdf, PipelineMarker},
    specialization::SdfPipeline,
    RenderPhase, SdfPipelineKey,
};
use bevy::{
    prelude::*,
    render::{
        render_phase::{DrawFunctions, ViewSortedRenderPhases},
        render_resource::{BufferUsages, PipelineCache, RawBufferVec, SpecializedRenderPipelines},
        sync_world::MainEntity,
        view::ExtractedView,
    },
};
use bytemuck::NoUninit;
use std::{marker::PhantomData, ops::Range};

pub(crate) fn queue_sdfs<P: RenderPhase>(
    sdfs: Query<(Entity, &MainEntity, &ExtractedRenderSdf), With<PipelineMarker<P>>>,
    views: Query<Entity, With<ExtractedView>>,
    sdf_pipeline: Res<SdfPipeline>,
    draw_functions: Res<DrawFunctions<P>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SdfPipeline>>,
    cache: Res<PipelineCache>,
    mut render_phases: ResMut<ViewSortedRenderPhases<P>>,
) {
    let draw_function = draw_functions.read().id::<DrawSdf>();
    for view_entity in views.into_iter() {
        let Some(render_phase) = render_phases.get_mut(&view_entity) else {
            continue;
        };
        for (entity, main_entity, sdf) in sdfs.iter() {
            let pipeline = pipelines.specialize(
                &cache,
                &sdf_pipeline,
                SdfPipelineKey {
                    pipeline: P::pipeline(),
                },
            );
            render_phase.add(P::phase_item(
                sdf.sort,
                (entity, *main_entity),
                pipeline,
                draw_function,
            ));
        }
    }
}

#[derive(Component, Debug)]
pub struct SdfBatch {
    pub range: Range<u32>,
}

#[derive(Debug, NoUninit, Clone, Copy)]
#[repr(C)]
pub struct SdfInstance {
    pos: Vec2,
    bounding_radius: f32,
    start_index: u32,
    op_count: u32,
}

#[derive(Resource)]
pub struct RenderPhaseBuffers<P: RenderPhase> {
    pub vertex: RawBufferVec<SdfInstance>,
    marker: PhantomData<P>,
}

impl<P: RenderPhase> Default for RenderPhaseBuffers<P> {
    fn default() -> Self {
        Self {
            vertex: RawBufferVec::new(BufferUsages::VERTEX),
            marker: PhantomData,
        }
    }
}

pub(crate) fn prepare_sdfs<P: RenderPhase>(
    mut cmds: Commands,
    mut phases: ResMut<ViewSortedRenderPhases<P>>,
    mut buffers: ResMut<RenderPhaseBuffers<P>>,
    sdfs: Query<&ExtractedRenderSdf>,
) {
    let mut batches = Vec::new();
    buffers.vertex.clear();

    for transparent_phase in phases.values_mut() {
        let mut batch_index = 0;
        let mut batch = false;

        for index in 0..transparent_phase.items.len() {
            let item = &transparent_phase.items[index];
            let Ok(sdf) = sdfs.get(item.entity()) else {
                batch = false;
                continue;
            };

            if !batch {
                batch = true;
                batch_index = index;
                let index = index as u32;
                batches.push((
                    item.entity(),
                    SdfBatch {
                        range: index..index,
                    },
                ));
            }

            let instance = SdfInstance {
                bounding_radius: sdf.final_bounds.circle.radius,
                pos: sdf.final_bounds.center,
                start_index: sdf.op_start_index,
                op_count: sdf.op_count,
            };

            trace_once!("{instance:#?}");

            buffers.vertex.push(instance);

            transparent_phase.items[batch_index].batch_range_mut().end += 1;
            batches.last_mut().unwrap().1.range.end += 1;
        }
    }

    cmds.insert_or_spawn_batch(batches);
}

pub(super) fn cleanup_batches(batches: Query<Entity, With<SdfBatch>>, mut cmds: Commands) {
    for entity in &batches {
        cmds.entity(entity).remove::<SdfBatch>();
    }
}
