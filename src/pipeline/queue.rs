use super::{
    draw::DrawSdf, specialization::CuttlePipeline, CuttlePipelineKey, SortedCuttlePhaseItem,
};
use crate::pipeline::extract::{ExtractedCuttle, ExtractedCuttles};
use bevy::render::sync_world::{MainEntity, TemporaryRenderEntity};
use bevy::{
    prelude::*,
    render::{
        render_phase::{DrawFunctions, ViewSortedRenderPhases},
        render_resource::{BufferUsages, PipelineCache, RawBufferVec, SpecializedRenderPipelines},
        view::ExtractedView,
    },
};
use bytemuck::NoUninit;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;

pub(crate) fn cuttle_queue_sorted_for_group<P: SortedCuttlePhaseItem>(
    mut cmds: Commands,
    items: Res<ExtractedCuttles>,
    views: Query<Entity, With<ExtractedView>>,
    cuttle_pipeline: Res<CuttlePipeline>,
    draw_functions: Res<DrawFunctions<P>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CuttlePipeline>>,
    cache: Res<PipelineCache>,
    mut render_phases: ResMut<ViewSortedRenderPhases<P>>,
) {
    let draw_function = draw_functions.read().id::<DrawSdf<P>>();
    for view_entity in views.into_iter() {
        let Some(render_phase) = render_phases.get_mut(&view_entity) else {
            continue;
        };
        for (
            &entity,
            &ExtractedCuttle {
                z,
                visible,
                group_id,
                ..
            },
        ) in items.iter()
        {
            if !visible {
                continue;
            }
            let pipeline = pipelines.specialize(
                &cache,
                &cuttle_pipeline,
                CuttlePipelineKey {
                    multisample_count: P::multisample_count(),
                    group_id,
                    has_depth: P::depth(),
                },
            );
            render_phase.add(P::phase_item(
                z,
                (
                    cmds.spawn(TemporaryRenderEntity).id(),
                    MainEntity::from(entity),
                ),
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
pub struct GroupInstanceBuffer<P> {
    pub vertex: RawBufferVec<CuttleInstance>,
    _phantom: PhantomData<P>,
}

impl<P> Default for GroupInstanceBuffer<P> {
    fn default() -> Self {
        Self {
            vertex: RawBufferVec::new(BufferUsages::VERTEX),
            _phantom: PhantomData,
        }
    }
}

pub(crate) fn cuttle_prepare_sorted_for_group<P: SortedCuttlePhaseItem>(
    mut cmds: Commands,
    mut phases: ResMut<ViewSortedRenderPhases<P>>,
    mut buffers: ResMut<GroupInstanceBuffer<P>>,
    items: Res<ExtractedCuttles>,
) {
    let mut batches = Vec::new();
    buffers.vertex.clear();

    for transparent_phase in phases.values_mut() {
        let mut batch_index = 0;
        let mut batch_z = f32::NAN;
        let mut batch = false;

        for index in 0..transparent_phase.items.len() {
            let item = &transparent_phase.items[index];
            let Some(&ExtractedCuttle {
                z,
                indices_start,
                indices_end,
                bounding,
                ..
            }) = items.get(&item.main_entity().id())
            else {
                batch = false;
                continue;
            };

            if !batch || batch_z != z {
                batch = true;
                batch_index = index;
                batch_z = z;
                let index = index as u32;
                batches.push((
                    item.entity(),
                    (
                        CuttleBatch {
                            range: index..index,
                        },
                        TemporaryRenderEntity,
                    ),
                ));
            }

            let instance = CuttleInstance {
                bounding_radius: bounding.circle.radius,
                pos: bounding.center,
                start: indices_start,
                end: indices_end,
            };

            buffers.vertex.push(instance);

            transparent_phase.items[batch_index].batch_range_mut().end += 1;
            batches.last_mut().unwrap().1 .0.range.end += 1;
        }
    }

    cmds.insert_or_spawn_batch(batches);
}
