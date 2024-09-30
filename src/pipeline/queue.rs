use super::{
    draw::DrawSdf, extract::ExtractedSdfs, specialization::SdfPipeline, RenderPhase, SdfPipelineKey,
};
use bevy::{
    prelude::*,
    render::{
        render_phase::{DrawFunctions, ViewSortedRenderPhases},
        render_resource::{
            BufferUsages, PipelineCache, RawBufferVec, ShaderType, SpecializedRenderPipelines,
        },
        view::ExtractedView,
    },
};
use bytemuck::NoUninit;
use std::{marker::PhantomData, ops::Range};

pub(crate) fn queue_sdfs<P: RenderPhase>(
    sdfs: Res<ExtractedSdfs<P>>,
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
        for (&entity, sdf) in sdfs.iter() {
            let pipeline = pipelines.specialize(&cache, &sdf_pipeline, sdf.key.clone());
            render_phase.add(P::phase_item(sdf.sort, entity, pipeline, draw_function));
        }
    }
}

#[derive(Component, Debug)]
pub struct SdfBatch {
    pub range: Range<u32>,
    pub key: SdfPipelineKey,
}

#[derive(Debug, ShaderType, NoUninit, Clone, Copy)]
#[repr(C)]
pub struct SdfInstance {
    size: Vec2,
    pos: Vec2,
    indices_start: u32,
}

#[derive(Resource)]
pub struct RenderPhaseBuffers<P: RenderPhase> {
    pub vertex: RawBufferVec<SdfInstance>,
    pub indices: RawBufferVec<u32>,
    marker: PhantomData<P>,
}

impl<P: RenderPhase> Default for RenderPhaseBuffers<P> {
    fn default() -> Self {
        Self {
            vertex: RawBufferVec::new(BufferUsages::VERTEX),
            indices: RawBufferVec::new(BufferUsages::STORAGE),
            marker: PhantomData,
        }
    }
}

pub(crate) fn prepare_sdfs<P: RenderPhase>(
    mut cmds: Commands,
    mut phases: ResMut<ViewSortedRenderPhases<P>>,
    mut buffers: ResMut<RenderPhaseBuffers<P>>,
    sdfs: Res<ExtractedSdfs<P>>,
) {
    let mut batches = Vec::new();
    buffers.indices.clear();
    buffers.vertex.clear();

    for transparent_phase in phases.values_mut() {
        let mut batch_index = 0;
        let mut batch_key = None;

        for index in 0..transparent_phase.items.len() {
            let item = &transparent_phase.items[index];
            let Some(sdf) = sdfs.get(&item.entity()) else {
                batch_key = None;
                continue;
            };

            if batch_key != Some(&sdf.key) {
                batch_index = index;
                batch_key = Some(&sdf.key);
                let index = index as u32;
                batches.push((
                    item.entity(),
                    SdfBatch {
                        key: sdf.key.clone(),
                        range: index..index,
                    },
                ));
            }

            let indices_start = buffers.indices.len() as u32;
            let instance = SdfInstance {
                size: sdf.aabb.size(),
                pos: sdf.aabb.pos(),
                indices_start,
            };

            buffers.vertex.push(instance);

            buffers.indices.extend(sdf.indices.iter().copied());

            transparent_phase.items[batch_index].batch_range_mut().end += 1;
            batches.last_mut().unwrap().1.range.end += 1;
        }
    }

    cmds.insert_or_spawn_batch(batches);
}
