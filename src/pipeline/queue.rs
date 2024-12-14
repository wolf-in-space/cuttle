use super::{
    draw::DrawSdf, extract::ExtractedRenderSdf, specialization::CuttlePipeline, RenderPhase,
    SdfPipelineKey,
};
use crate::groups::GroupId;
use crate::pipeline::extract::{ExtractedCuttleTransform, ExtractedVisibility};
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::render::render_phase::PhaseItem;
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
use std::ops::Range;

pub(crate) fn queue_sdfs(
    sdfs: Query<
        (
            Entity,
            &MainEntity,
            &ExtractedVisibility,
            &ExtractedCuttleTransform,
        ),
        With<ExtractedRenderSdf>,
    >,
    views: Query<Entity, With<ExtractedView>>,
    sdf_pipeline: Res<CuttlePipeline>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CuttlePipeline>>,
    cache: Res<PipelineCache>,
    mut render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
) {
    let draw_function = draw_functions.read().id::<DrawSdf>();
    for view_entity in views.into_iter() {
        let Some(render_phase) = render_phases.get_mut(&view_entity) else {
            continue;
        };
        for (entity, main_entity, visibility, transform) in sdfs.iter() {
            if !visibility.0 {
                continue;
            }
            let pipeline = pipelines.specialize(
                &cache,
                &sdf_pipeline,
                SdfPipelineKey {
                    group_id: GroupId(0),
                },
            );
            render_phase.add(Transparent2d::phase_item(
                transform.z,
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
pub struct RenderPhaseBuffers {
    pub vertex: RawBufferVec<SdfInstance>,
}

impl Default for RenderPhaseBuffers {
    fn default() -> Self {
        Self {
            vertex: RawBufferVec::new(BufferUsages::VERTEX),
        }
    }
}

pub(crate) fn prepare_sdfs(
    mut cmds: Commands,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    mut buffers: ResMut<RenderPhaseBuffers>,
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
