use super::{RenderPhase, SdfPipelineKey};
use crate::{
    aabb::CombinedAABB, builder::RenderSdf, components::extract::SdfBufferIndices, flag::SdfFlags,
};
use bevy::{ecs::entity::EntityHashMap, prelude::*, render::Extract};
use bevy_comdf_core::aabb::AABB;
use std::marker::PhantomData;

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

pub(crate) fn extract_render_sdf<P: RenderPhase>(
    query: Extract<
        Query<(
            Entity,
            &RenderSdf<P>,
            &SdfBufferIndices,
            &CombinedAABB,
            &SdfFlags,
            &GlobalTransform,
        )>,
    >,
    mut extracted: ResMut<ExtractedSdfs<P>>,
) {
    **extracted = query
        .into_iter()
        .map(|(entity, render, indices, aabb, flags, tranform)| {
            (
                entity,
                ExtractedSdf {
                    key: SdfPipelineKey {
                        pipeline: render.pipeline,
                        flags: flags.clone(),
                    },
                    aabb: aabb.0.clone(),
                    indices: indices.0.clone(),
                    sort: tranform.translation().z,
                },
            )
        })
        .collect();
}
