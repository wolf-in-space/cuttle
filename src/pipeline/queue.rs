use super::{
    draw::DrawCuttle, specialization::CuttlePipeline, CuttlePipelineKey, SortedCuttlePhaseItem,
};
use crate::components::buffer::ConfigRenderEntity;
use crate::groups::{ConfigId, CuttleConfig};
use crate::pipeline::extract::{Extracted, ExtractedCuttle};
use bevy::render::render_phase::PhaseItem;
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

pub fn cuttle_queue_sorted_for_group<Config: CuttleConfig>(
    mut cmds: Commands,
    extracted: Single<&Extracted, With<ConfigRenderEntity<Config>>>,
    views: Query<Entity, With<ExtractedView>>,
    cuttle_pipeline: Res<CuttlePipeline>,
    draw_functions: Res<DrawFunctions<Config::Phase>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CuttlePipeline>>,
    cache: Res<PipelineCache>,
    mut render_phases: ResMut<ViewSortedRenderPhases<Config::Phase>>,
) {
    let draw_function = draw_functions.read().id::<DrawCuttle<Config>>();
    for view_entity in views.into_iter() {
        let Some(render_phase) = render_phases.get_mut(&view_entity) else {
            continue;
        };
        for (
            &entity,
            &ExtractedCuttle {
                z,
                visible,
                group_id: item_group_id,
                ..
            },
        ) in extracted.iter()
        {
            if !visible {
                continue;
            }
            let pipeline = pipelines.specialize(
                &cache,
                &cuttle_pipeline,
                CuttlePipelineKey {
                    multisample_count: Config::Phase::multisample_count(),
                    group_id: ConfigId(item_group_id),
                    has_depth: Config::Phase::depth(),
                },
            );
            render_phase.add(Config::Phase::phase_item(
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

pub fn cuttle_prepare_sorted_for_group<Config: CuttleConfig>(
    mut cmds: Commands,
    mut phases: ResMut<ViewSortedRenderPhases<Config::Phase>>,
    mut buffers: ResMut<ConfigInstanceBuffer<Config>>,
    extracted: Single<&Extracted, With<ConfigRenderEntity<Config>>>,
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
            }) = extracted.get(&item.main_entity().id())
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
