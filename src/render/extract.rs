use super::shader::buffers::{SdfOperationsBuffer, SdfStorageBuffer};
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
    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
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
    query: Extract<Query<Entity, Or<(With<Sdf>, With<RenderSdf>)>>>,
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
    // println!("extracting '{}' sdfs", query.iter().len());
    cmds.insert_or_spawn_batch(
        query
            .into_iter()
            .map(|(e, aabb)| {
                (
                    *translator.0.get(&e).unwrap(),
                    (
                        aabb.clone(),
                        SdfStorageBuffer::default(),
                        RenderableSdf::default(),
                        SdfStorageIndex::default(),
                    ),
                )
            })
            .collect_vec(),
    )
}

fn extract_render_sdfs(
    mut cmds: Commands,
    translator: Res<EntityTranslator>,
    query: Extract<Query<Entity, With<RenderSdf>>>,
) {
    // println!("extracting '{}' render sdfs", query.iter().len());
    cmds.insert_or_spawn_batch(
        query
            .into_iter()
            .map(|e| {
                (
                    *translator.0.get(&e).unwrap(),
                    (
                        SdfPipelineKey::default(),
                        SdfOperationsBuffer::default(),
                        AABB::default(),
                    ),
                )
            })
            .collect_vec(),
    )
}
