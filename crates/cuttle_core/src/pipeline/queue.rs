use super::{
    draw::DrawCuttle, specialization::CuttlePipeline, CuttlePipelineKey, SortedCuttlePhaseItem,
};
use crate::components::buffer::ConfigRenderEntity;
use crate::configs::{ConfigId, CuttleConfig};
use crate::internal_prelude::*;
use crate::pipeline::extract::{Extracted, ExtractedCuttle};
use bevy_math::Vec2;
use bevy_platform::collections::HashMap;
use bevy_render::render_phase::{DrawFunctions, PhaseItem, ViewSortedRenderPhases};
use bevy_render::render_resource::{
    BufferUsages, PipelineCache, RawBufferVec, SpecializedRenderPipelines,
};
use bevy_render::sync_world::MainEntity;
use bevy_render::view::{ExtractedView, RetainedViewEntity};
use bytemuck::NoUninit;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;

pub fn cuttle_queue_sorted_for_config<Config: CuttleConfig>(
    extracted: Single<&Extracted, With<ConfigRenderEntity<Config>>>,
    views: Query<&ExtractedView>,
    cuttle_pipeline: Res<CuttlePipeline>,
    draw_functions: Res<DrawFunctions<Config::Phase>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CuttlePipeline>>,
    cache: Res<PipelineCache>,
    mut render_phases: ResMut<ViewSortedRenderPhases<Config::Phase>>,
) {
    let draw_function = draw_functions.read().id::<DrawCuttle<Config>>();
    for view in views.into_iter() {
        let Some(render_phase) = render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        for (index, (&entity, cuttle)) in extracted.iter().enumerate() {
            let &ExtractedCuttle {
                z,
                render_entity,
                group_id,
                ..
            } = cuttle;
            let pipeline = pipelines.specialize(
                &cache,
                &cuttle_pipeline,
                CuttlePipelineKey {
                    multisample_count: Config::Phase::multisample_count(),
                    group_id: ConfigId(group_id),
                    has_depth: Config::Phase::depth(),
                },
            );
            render_phase.add(Config::Phase::phase_item(
                index,
                z,
                (render_entity, MainEntity::from(entity)),
                pipeline,
                draw_function,
            ));
        }
    }
}

#[derive(Debug)]
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
pub struct ConfigInstanceBuffer<Config: CuttleConfig> {
    pub vertex: RawBufferVec<CuttleInstance>,
    _phantom: PhantomData<Config>,
}

impl<Config: CuttleConfig> Default for ConfigInstanceBuffer<Config> {
    fn default() -> Self {
        Self {
            vertex: RawBufferVec::new(BufferUsages::VERTEX),
            _phantom: PhantomData,
        }
    }
}

#[derive(Default, Resource, Debug, Deref, DerefMut)]
pub struct CuttleBatches(pub HashMap<(RetainedViewEntity, Entity), CuttleBatch>);

pub fn cuttle_prepare_sorted_for_config<Config: CuttleConfig>(
    mut phases: ResMut<ViewSortedRenderPhases<Config::Phase>>,
    mut buffers: ResMut<ConfigInstanceBuffer<Config>>,
    mut batches: ResMut<CuttleBatches>,
    extracted: Single<&Extracted, With<ConfigRenderEntity<Config>>>,
) {
    buffers.vertex.clear();
    batches.clear();

    for (retained_view, phase) in phases.iter_mut() {
        let mut batch_index = 0;
        let mut batch_z = f32::NAN;
        let mut batch = None;

        for index in 0..phase.items.len() {
            let item = &phase.items[index];
            let Some(&ExtractedCuttle {
                z,
                indices_start,
                indices_end,
                bounding,
                ..
            }) = extracted.get(&item.main_entity().id())
            else {
                batch = None;
                continue;
            };

            if batch.is_none() || batch_z != z {
                batch_index = index;
                batch_z = z;
                let index = index as u32;
                batch = Some(batches.entry((*retained_view, item.entity())).or_insert(
                    CuttleBatch {
                        range: index..index,
                    },
                ));
            }

            let instance = CuttleInstance {
                bounding_radius: bounding.circle.radius,
                pos: bounding.center,
                start: indices_start,
                end: indices_end,
            };

            buffers.vertex.push(instance);

            phase.items[batch_index].batch_range_mut().end += 1;
            batch.as_mut().unwrap().range.end += 1;
        }
    }
}
