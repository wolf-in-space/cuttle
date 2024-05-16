use super::shader::buffers::SdfStorageBuffer;
use crate::scheduling::ComdfRenderSet::*;
use crate::{
    flag::{RenderableSdf, SdfPipelineKey},
    prelude::SdfStorageIndex,
    RenderSdf,
};
use bevy_app::prelude::*;
use bevy_comdf_core::{aabb::AABB, prepare::Sdf};
use bevy_ecs::{entity::EntityHashMap, prelude::*};
use bevy_render::{Extract, ExtractSchedule, RenderApp};
use itertools::Itertools;

pub fn plugin(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);
    render_app.init_resource::<EntityTranslator>();
    render_app.add_systems(
        ExtractSchedule,
        (
            setup_entity_translation.before(PrepareExtract),
            (extract_render_sdfs, extract_sdf_entitys).in_set(Extract),
        ),
    );
}

#[derive(Default, Resource)]
pub struct EntityTranslator(pub EntityHashMap<Entity>);

fn setup_entity_translation(
    mut cmds: Commands,
    mut translator: ResMut<EntityTranslator>,
    query: Extract<Query<Entity, With<Sdf>>>,
) {
    translator.0.clear();
    for entity in query.into_iter() {
        let render_entity = cmds.spawn_empty().id();
        translator.0.insert(entity, render_entity);
    }
}

fn extract_sdf_entitys(
    mut cmds: Commands,
    translator: Res<EntityTranslator>,
    query: Extract<Query<(Entity, &AABB), With<Sdf>>>,
) {
    cmds.insert_or_spawn_batch(
        query
            .into_iter()
            .filter_map(|(e, aabb)| {
                Some((
                    *translator.0.get(&e)?,
                    (
                        aabb.clone(),
                        SdfPipelineKey::default(),
                        SdfStorageBuffer::default(),
                        RenderableSdf::default(),
                        SdfStorageIndex::default(),
                    ),
                ))
            })
            .collect_vec(),
    )
}

fn extract_render_sdfs(
    mut cmds: Commands,
    translator: Res<EntityTranslator>,
    query: Extract<Query<Entity, With<RenderSdf>>>,
) {
    cmds.insert_or_spawn_batch(
        query
            .into_iter()
            .filter_map(|e| {
                Some((
                    *translator.0.get(&e)?,
                    (RenderSdf, SdfPipelineKey::default()),
                ))
            })
            .collect_vec(),
    )
}
