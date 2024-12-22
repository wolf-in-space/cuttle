use super::{
    draw::DrawSdf, specialization::CuttlePipeline, RenderPhase,
    SdfPipelineKey,
};
use crate::groups::CuttleGroup;
use crate::pipeline::extract::{CombinedBounding, ExtractedVisibility, ExtractedZ, RenderIndexRange};
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
use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::Range;

pub(crate) fn cuttle_queue_sorted_for_group<G: CuttleGroup>(
    entities: Query<
        (
            Entity,
            &MainEntity,
            &ExtractedVisibility,
            &ExtractedZ,
        ),
        With<G>,
    >,
    views: Query<Entity, With<ExtractedView>>,
    sdf_pipeline: Res<CuttlePipeline>,
    draw_functions: Res<DrawFunctions<G::Phase>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CuttlePipeline>>,
    cache: Res<PipelineCache>,
    mut render_phases: ResMut<ViewSortedRenderPhases<G::Phase>>,
) {
    let draw_function = draw_functions.read().id::<DrawSdf<G>>();
    for view_entity in views.into_iter() {
        let Some(render_phase) = render_phases.get_mut(&view_entity) else {
            continue;
        };
        for (entity, main_entity, visibility, z) in entities.iter() {
            if !visibility.0 {
                continue;
            }
            let pipeline = pipelines.specialize(
                &cache,
                &sdf_pipeline,
                SdfPipelineKey {
                    multisample_count: G::Phase::multisample_count(),
                    group_id: TypeId::of::<G>(),
                    has_depth: G::Phase::depth(),
                },
            );
            render_phase.add(G::Phase::phase_item(
                z.0,
                (entity, *main_entity),
                pipeline,
                draw_function,
            ));
        }
    }
}

#[derive(Component, Debug)]
pub struct CuttleBatch {
    pub range: Range<u32>,
}

#[derive(Debug, NoUninit, Clone, Copy)]
#[repr(C)]
pub struct CuttleInstance {
    pos: Vec2,
    bounding_radius: f32,
    start: u32,
    end: u32,
}

#[derive(Resource)]
pub struct GroupBuffers<G: CuttleGroup> {
    pub vertex: RawBufferVec<CuttleInstance>,
    _phantom: PhantomData<G>,
}

impl<G: CuttleGroup> Default for GroupBuffers<G> {
    fn default() -> Self {
        Self {
            vertex: RawBufferVec::new(BufferUsages::VERTEX),
            _phantom: PhantomData,
        }
    }
}

pub(crate) fn cuttle_prepare_sorted_for_group<G: CuttleGroup>(
    mut cmds: Commands,
    mut phases: ResMut<ViewSortedRenderPhases<G::Phase>>,
    mut buffers: ResMut<GroupBuffers<G>>,
    entities: Query<(&CombinedBounding, &RenderIndexRange), With<G>>,
) {
    let mut batches = Vec::new();
    buffers.vertex.clear();

    for transparent_phase in phases.values_mut() {
        let mut batch_index = 0;
        let mut batch = false;

        for index in 0..transparent_phase.items.len() {
            let item = &transparent_phase.items[index];
            let Ok((bounds, range)) = entities.get(item.entity()) else {
                batch = false;
                continue;
            };

            if !batch {
                batch = true;
                batch_index = index;
                let index = index as u32;
                batches.push((
                    item.entity(),
                    CuttleBatch {
                        range: index..index,
                    },
                ));
            }

            let instance = CuttleInstance {
                bounding_radius: bounds.circle.radius,
                pos: bounds.center,
                start: range.start,
                end: range.end,
            };

            buffers.vertex.push(instance);

            transparent_phase.items[batch_index].batch_range_mut().end += 1;
            batches.last_mut().unwrap().1.range.end += 1;
        }
    }

    cmds.insert_or_spawn_batch(batches);
}

pub(super) fn cleanup_batches(batches: Query<Entity, With<CuttleBatch>>, mut cmds: Commands) {
    for entity in &batches {
        cmds.entity(entity).remove::<CuttleBatch>();
    }
}
