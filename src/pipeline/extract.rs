use std::marker::PhantomData;

use super::{SdfPipelineKey, UsePipeline};
use crate::{
    aabb::CombinedAABB, builder::RenderSdf, components::extract::SdfBufferIndices, flag::SdfFlags,
};
use bevy::{
    core_pipeline::core_2d::Transparent2d, ecs::entity::EntityHashMap, prelude::*, render::Extract,
    ui::TransparentUi,
};
use bevy_comdf_core::aabb::AABB;

#[derive(Debug)]
pub struct ExtractedSdf {
    pub indices: Vec<u32>,
    pub aabb: AABB,
    pub key: SdfPipelineKey,
    pub sort: f32,
}

#[derive(Resource, Deref, DerefMut)]
pub(crate) struct ExtractedSdfs<P> {
    #[deref]
    sdfs: EntityHashMap<ExtractedSdf>,
    marker: PhantomData<P>,
}

impl<P> Default for ExtractedSdfs<P> {
    fn default() -> Self {
        Self {
            sdfs: default(),
            marker: PhantomData,
        }
    }
}

pub(crate) fn extract_render_sdf(
    query: Extract<
        Query<(
            Entity,
            &RenderSdf,
            &SdfBufferIndices,
            &CombinedAABB,
            &SdfFlags,
            &GlobalTransform,
        )>,
    >,
    mut extracted_2d: ResMut<ExtractedSdfs<Transparent2d>>,
    mut extracted_ui: ResMut<ExtractedSdfs<TransparentUi>>,
) {
    extracted_2d.clear();
    extracted_ui.clear();

    for (entity, render, indices, aabb, flags, tranform) in &query {
        let extracted = ExtractedSdf {
            key: SdfPipelineKey {
                pipeline: render.pipeline,
                flags: flags.clone(),
            },
            aabb: aabb.0.clone(),
            indices: indices.0.clone(),
            sort: tranform.translation().z,
        };

        match render.pipeline {
            UsePipeline::World => extracted_2d.insert(entity, extracted),
            UsePipeline::Ui => extracted_ui.insert(entity, extracted),
        };
    }
}
