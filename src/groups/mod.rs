use crate::calculations::*;
use crate::components::buffer::ConfigRenderEntity;
use crate::components::ComponentInfos;
use crate::indices::{on_add_group_marker_initialize_indices_group_id, CuttleIndices};
use crate::pipeline::draw::DrawCuttle;
use crate::pipeline::extract::extract_cuttles;
use crate::pipeline::queue::{
    cuttle_prepare_sorted_for_group, cuttle_queue_sorted_for_group, ConfigInstanceBuffer,
};
use crate::pipeline::specialization::write_group_buffer;
use crate::pipeline::CuttleRenderSet::WriteBuffers;
use crate::pipeline::{CuttleRenderSet, SortedCuttlePhaseItem};
use crate::shader::Snippets;
use bevy::prelude::*;
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::sync_world::RenderEntity;
use bevy::render::{Render, RenderApp};
use global::GlobalGroupInfos;
use std::marker::PhantomData;

pub mod builder;
pub mod global;

pub trait CuttleConfig: Component + Default {
    type Phase: SortedCuttlePhaseItem;
}

fn initialize_config<Config: CuttleConfig>(app: &mut App) -> Entity {
    if let Some(store) = app.world().get_resource::<ConfigStore<Config>>() {
        return store.group;
    };

    if !app.world().contains_resource::<GlobalGroupInfos>() {
        let infos = GlobalGroupInfos::new(app);
        app.insert_resource(infos);
    }

    app.register_required_components::<Config, CuttleIndices>();
    app.world_mut()
        .register_component_hooks::<Config>()
        .on_add(on_add_group_marker_initialize_indices_group_id::<Config>);

    let config_id = initialize_config_id(app);
    let config_buffer_entity = initialize_config_render_world::<Config>(app, config_id);
    let initial_calculations = vec![
        // Always needed as the input to the fragment shader.
        Calculation::new("vertex", "VertexOut"),
        // Always needed as the output to the fragment shader.
        Calculation::new("color", "vec4<f32>"),
    ];

    let config_entity = app
        .world_mut()
        .spawn((
            config_id,
            Snippets::default(),
            ComponentInfos::default(),
            Calculations(initial_calculations),
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
                cuttle_queue_sorted_for_group::<Config>.in_set(CuttleRenderSet::Queue),
                cuttle_prepare_sorted_for_group::<Config>.in_set(CuttleRenderSet::ItemPreparation),
                write_group_buffer::<Config>.in_set(WriteBuffers),
            ),
        );

    config_buffer_entity
}

fn initialize_config_id(app: &mut App) -> ConfigId {
    let world = app.world_mut();
    let mut global = world.resource_mut::<GlobalGroupInfos>();
    let id = global.group_count;
    global.group_count += 1;
    global.component_positions.push(default());
    ConfigId(id)
}

#[derive(Debug, Copy, Clone, Component, Reflect, Hash, Eq, PartialEq)]
#[reflect(Component)]
pub struct ConfigId(pub(crate) usize);

#[derive(Resource)]
pub struct ConfigStore<G> {
    pub id: usize,
    pub group: Entity,
    phantom_data: PhantomData<G>,
}
impl<G> Copy for ConfigStore<G> {}
impl<G> Clone for ConfigStore<G> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            group: self.group,
            phantom_data: PhantomData,
        }
    }
}

impl<Config: CuttleConfig> ConfigStore<Config> {
    fn new(id: usize, entity: Entity) -> Self {
        Self {
            id,
            group: entity,
            phantom_data: PhantomData,
        }
    }
}
