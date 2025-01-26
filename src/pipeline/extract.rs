use crate::bounding::GlobalBoundingCircle;
use crate::components::arena::IndexArena;
use crate::components::buffer::{CompBuffer, ConfigRenderEntity, GlobalBuffer};
use crate::components::initialization::CuttleRenderData;
use crate::extensions::CompIndicesBuffer;
use crate::groups::{ConfigId, CuttleConfig};
use crate::indices::{CuttleComponentIndex, CuttleIndices};
use bevy::ecs::entity::EntityHashMap;
use bevy::render::{Render, RenderSet};
use bevy::{
    math::bounding::BoundingCircle,
    prelude::*,
    render::{Extract, RenderApp},
};
use std::fmt::Debug;
use std::ops::Deref;

pub fn plugin(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_systems(Render, clear_cuttles.in_set(RenderSet::Cleanup));
}

pub(crate) fn extract_cuttle_global<C: Component, R: CuttleRenderData>(
    mut buffer: Single<&mut GlobalBuffer<C, R>>,
    component: Extract<Option<Single<&C, (Changed<C>, With<ConfigId>)>>>,
) {
    if let Some(component) = component.deref() {
        buffer.set(&component);
    }
}

pub(crate) fn extract_cuttle_comp<C: Component, R: CuttleRenderData>(
    mut buffer: Single<&mut CompBuffer<C, R>>,
    arena: Extract<Res<IndexArena<C>>>,
    comps: Extract<Query<(&CuttleComponentIndex<C>, &C), Changed<C>>>,
) {
    buffer.resize(arena.max as usize);
    for (index, comp) in &comps {
        buffer.insert(**index as usize, comp);
    }
}

#[derive(Debug, Component, Default, Deref, DerefMut)]
pub struct Extracted(EntityHashMap<ExtractedCuttle>);

#[derive(Debug)]
pub struct ExtractedCuttle {
    pub group_id: usize,
    pub visible: bool,
    pub bounding: BoundingCircle,
    pub indices_start: u32,
    pub indices_end: u32,
    pub z: f32,
}

pub fn extract_cuttles<Config: CuttleConfig>(
    extract: Extract<
        Query<
            (
                Entity,
                Option<&GlobalTransform>,
                &GlobalBoundingCircle,
                &CuttleIndices,
                &ViewVisibility,
            ),
            With<Config>,
        >,
    >,
    mut buffer: ResMut<CompIndicesBuffer>,
    mut extracted: Single<&mut Extracted, With<ConfigRenderEntity<Config>>>,
) {
    let buffer = buffer.get_mut();

    extracted.extend(
        extract
            .iter()
            .map(|(entity, transform, bounding, indices, vis)| {
                let indices_start = buffer.len() as u32;
                let indices_end = (buffer.len() + indices.indices.len()) as u32;
                buffer.extend(indices.iter_as_packed_u32s());

                (
                    entity,
                    ExtractedCuttle {
                        group_id: indices.group_id,
                        visible: **vis,
                        indices_start,
                        indices_end,
                        bounding: **bounding,
                        z: transform.map(|t| t.translation().z).unwrap_or_default(),
                    },
                )
            }),
    );
}

fn clear_cuttles(mut extracted: Query<&mut Extracted>, mut buffer: ResMut<CompIndicesBuffer>) {
    for mut extracted in &mut extracted {
        extracted.clear()
    }
    buffer.get_mut().clear();
}
