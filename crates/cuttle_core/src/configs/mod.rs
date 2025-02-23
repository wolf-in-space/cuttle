use crate::components::buffer::ConfigRenderEntity;
use crate::components::ConfigComponents;
use crate::indices::{on_add_config_marker_initialize_indices_config_id, CuttleIndices};
use crate::internal_prelude::*;
use crate::pipeline::draw::DrawCuttle;
use crate::pipeline::extract::extract_cuttles;
use crate::pipeline::queue::{
    cuttle_prepare_sorted_for_config, cuttle_queue_sorted_for_config, ConfigInstanceBuffer,
};
use crate::pipeline::specialization::write_group_buffer;
use crate::pipeline::CuttleRenderSet::WriteBuffers;
use crate::pipeline::{CuttleRenderSet, SortedCuttlePhaseItem};
use crate::shader::Snippets;
use bevy_core::Name;
use bevy_render::render_phase::AddRenderCommand;
use bevy_render::sync_world::RenderEntity;
use bevy_render::{Render, RenderApp};
use global::GlobalConfigInfos;
use std::any::type_name;
use std::marker::PhantomData;

pub mod builder;
pub mod global;

pub trait CuttleConfig: Component + Default {
    type Phase: SortedCuttlePhaseItem;
}

fn initialize_config<Config: CuttleConfig>(app: &mut App) -> Entity {
    if let Some(store) = app.world().get_resource::<ConfigStore<Config>>() {
        return store.config_entity;
    };

    if !app.world().contains_resource::<GlobalConfigInfos>() {
        let infos = GlobalConfigInfos::new(app);
        app.insert_resource(infos);
    }

    app.register_required_components::<Config, CuttleIndices>();
    app.world_mut()
        .register_component_hooks::<Config>()
        .on_add(on_add_config_marker_initialize_indices_config_id::<Config>);

    let config_id = initialize_config_id(app);
    let config_buffer_entity = initialize_config_render_world::<Config>(app, config_id);

    let config_entity = app
        .world_mut()
        .spawn((
            Name::new(format!("Cuttle Config for {}", type_name::<Config>())),
            config_id,
            ConfigComponents::default(),
            Snippets::default(),
            RenderEntity::from(config_buffer_entity),
        ))
        .id();

    app.world_mut()
        .insert_resource(ConfigStore::<Config>::new(config_id.0, config_entity));

    config_entity
}

fn initialize_config_render_world<Config: CuttleConfig>(
    app: &mut App,
    config_id: ConfigId,
) -> Entity {
    let render_app = app.sub_app_mut(RenderApp);
    let config_buffer_entity = render_app
        .world_mut()
        .spawn((ConfigRenderEntity::<Config>::new(), config_id))
        .id();

    render_app
        .add_render_command::<Config::Phase, DrawCuttle<Config>>()
        .init_resource::<ConfigInstanceBuffer<Config>>()
        .add_systems(ExtractSchedule, extract_cuttles::<Config>)
        .add_systems(
            Render,
            (
                cuttle_queue_sorted_for_config::<Config>.in_set(CuttleRenderSet::Queue),
                cuttle_prepare_sorted_for_config::<Config>.in_set(CuttleRenderSet::ItemPreparation),
                write_group_buffer::<Config>.in_set(WriteBuffers),
            ),
        );

    config_buffer_entity
}

fn initialize_config_id(app: &mut App) -> ConfigId {
    let world = app.world_mut();
    let mut global = world.resource_mut::<GlobalConfigInfos>();
    let id = global.config_count;
    global.config_count += 1;
    ConfigId(id)
}

#[derive(Debug, Copy, Clone, Component, Reflect, Hash, Eq, PartialEq)]
#[reflect(Component)]
pub struct ConfigId(pub(crate) usize);

#[derive(Resource)]
pub struct ConfigStore<Config> {
    pub id: usize,
    pub config_entity: Entity,
    phantom_data: PhantomData<Config>,
}
impl<Config> Copy for ConfigStore<Config> {}
impl<Config> Clone for ConfigStore<Config> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            config_entity: self.config_entity,
            phantom_data: PhantomData,
        }
    }
}

impl<Config: CuttleConfig> ConfigStore<Config> {
    fn new(id: usize, entity: Entity) -> Self {
        Self {
            id,
            config_entity: entity,
            phantom_data: PhantomData,
        }
    }
}
