use crate::bounding::GlobalBoundingCircle;
use crate::components::arena::IndexArena;
use crate::components::buffer::{CompBuffer, ConfigRenderEntity, GlobalBuffer};
use crate::components::initialization::CuttleRenderData;
use crate::configs::{ConfigId, CuttleConfig};
use crate::extensions::CompIndicesBuffer;
use crate::indices::{CuttleComponentIndex, CuttleIndices};
use crate::internal_prelude::*;
use bevy_app::{App, PostUpdate};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::entity::hash_map::EntityHashMap;
use bevy_math::bounding::BoundingCircle;
use bevy_render::sync_world::RenderEntity;
use bevy_render::{Extract, Render, RenderApp, RenderSet};
use bevy_transform::TransformSystem;
use std::fmt::Debug;
use std::ops::Deref;

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        set_cuttle_z_from_bevy_global_transform.after(TransformSystem::TransformPropagate),
    );
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
        // info!("Extracting {} at index {}", type_name::<C>(), **index);
        buffer.insert(**index as usize, comp);
    }
}

#[derive(Debug, Default, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct CuttleZ(pub f32);

pub fn set_cuttle_z_from_bevy_global_transform(mut query: Query<(&mut CuttleZ, &GlobalTransform)>) {
    for (mut z, transform) in &mut query {
        z.0 = transform.translation().z;
    }
}

#[derive(Debug, Component, Default, Deref, DerefMut)]
pub struct Extracted(EntityHashMap<ExtractedCuttle>);

#[derive(Debug)]
pub struct ExtractedCuttle {
    pub render_entity: Entity,
    pub group_id: usize,
    pub bounding: BoundingCircle,
    pub indices_start: u32,
    pub indices_end: u32,
    pub z: f32,
}

pub fn extract_cuttles<Config: CuttleConfig>(
    extract: Extract<
        Query<
            (
                &ViewVisibility,
                Entity,
                RenderEntity,
                &CuttleZ,
                &GlobalBoundingCircle,
                &CuttleIndices,
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
            //.filter(|(visibility, ..)| visibility.get())
            .map(
                |(_, entity, render_entity, &CuttleZ(z), bounding, indices)| {
                    let indices_start = buffer.len() as u32;
                    let indices_end = (buffer.len() + indices.indices.len()) as u32;
                    let indices_iter = indices.iter_as_packed_u32s();

                    #[cfg(feature = "debug")]
                    {}

                    buffer.extend(indices_iter);

                    (
                        entity,
                        ExtractedCuttle {
                            render_entity,
                            group_id: indices.group_id,
                            indices_start,
                            indices_end,
                            bounding: **bounding,
                            z,
                        },
                    )
                },
            ),
    );
}

fn clear_cuttles(mut extracted: Query<&mut Extracted>, mut buffer: ResMut<CompIndicesBuffer>) {
    for mut extracted in &mut extracted {
        extracted.clear()
    }
    buffer.get_mut().clear();
}
