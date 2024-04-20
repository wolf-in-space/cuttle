use super::{building::SdfShaderBuilder, lines::Lines, variants::VariantShaderBuilder};
use crate::render::pipeline::{SdfPipeline, SdfSpecializationData};
use crate::scheduling::ComdfRenderSet::*;
use crate::{
    flag::{RenderableSdf, SdfPipelineKey, VariantFlag},
    operations::OperationsFlag,
};
use bevy_app::prelude::*;
use bevy_asset::AssetServer;
use bevy_ecs::prelude::*;
use bevy_log::error;
use bevy_render::render_resource::{BindGroupEntry, BufferUsages, BufferVec};
use bevy_render::renderer::RenderDevice;
use bevy_render::{Render, RenderApp};
use bevy_utils::{HashMap, HashSet};
use itertools::Itertools;

pub fn plugin(app: &mut App) {
    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app.add_event::<NewSdfFlag>();
    render_app.add_event::<NewPipelineKey>();
    render_app.add_systems(
        Render,
        (
            assign_bindings_and_trigger_new_sdf_events.in_set(AssignSdfBindings),
            create_new_bind_group_buffers
                .after(AssignSdfBindings)
                .before(PrepareBatches),
            trigger_new_key_events
                .after(BuildPipelineKeys)
                .before(PrepareShaderBuild),
            (prepare_sdf_shader_build, prepare_render_shader_build).in_set(PrepareShaderBuild),
            (gather_loaded_calculations, build_new_shaders)
                .chain()
                .in_set(CollectShaders),
            build_sdf_bindgroups.in_set(BuildBindgroups),
        ),
    );
}

#[derive(Resource, Default)]
pub struct SdfShaderRegister {
    pub snippets: HashMap<OperationsFlag, Lines>,
    pub bindings: HashMap<VariantFlag, u32>,
    loaded_calcs: HashMap<u32, VariantShaderBuilder>,
    shaders: HashSet<SdfPipelineKey>,
    loaded_shaders: HashSet<SdfPipelineKey>,
}

#[derive(Event)]
pub struct NewSdfFlag {
    entity: Entity,
    binding: u32,
}

pub fn assign_bindings_and_trigger_new_sdf_events(
    mut query: Query<(Entity, &mut RenderableSdf)>,
    mut registry: ResMut<SdfShaderRegister>,
    mut new_sdfs: EventWriter<NewSdfFlag>,
) {
    // println!("assign_bindings: {}", query.iter().len());
    query.iter_mut().for_each(|(entity, mut renderable)| {
        if let Some(binding) = registry.bindings.get(&renderable.flag) {
            renderable.binding = *binding;
        } else {
            let binding = registry.bindings.len() as u32;
            registry.bindings.insert(renderable.flag, binding);
            new_sdfs.send(NewSdfFlag { entity, binding });
            renderable.binding = binding;
        }
    });
}

pub fn prepare_sdf_shader_build(mut cmds: Commands, mut new_sdfs: EventReader<NewSdfFlag>) {
    new_sdfs.read().for_each(|new| {
        cmds.entity(new.entity)
            .insert(VariantShaderBuilder::new(new.binding));
    })
}

pub fn create_new_bind_group_buffers(
    mut pipeline: ResMut<SdfPipeline>,
    mut new_sdfs: EventReader<NewSdfFlag>,
) {
    new_sdfs.read().for_each(|_| {
        let buffer = BufferVec::new(BufferUsages::STORAGE);
        pipeline.bind_group_buffers.push(buffer);
    })
}

pub fn prepare_render_shader_build(mut cmds: Commands, mut new_sdfs: EventReader<NewPipelineKey>) {
    new_sdfs.read().for_each(|new| {
        cmds.entity(new.entity).insert(SdfShaderBuilder::new());
    })
}

#[derive(Event)]
pub struct NewPipelineKey {
    entity: Entity,
    key: SdfPipelineKey,
}

pub fn trigger_new_key_events(
    query: Query<(Entity, &SdfPipelineKey)>,
    mut shaders: ResMut<SdfShaderRegister>,
    mut new_keys: EventWriter<NewPipelineKey>,
) {
    for (entity, key) in query.iter() {
        if shaders.shaders.contains(key) {
            continue;
        }
        shaders.shaders.insert(key.clone());
        new_keys.send(NewPipelineKey {
            entity,
            key: key.clone(),
        });
    }
}

pub fn gather_loaded_calculations(
    mut cmds: Commands,
    query: Query<(Entity, &RenderableSdf, &VariantShaderBuilder)>,
    mut shaders: ResMut<SdfShaderRegister>,
) {
    query.iter().for_each(|(entity, variant, calculation)| {
        shaders
            .loaded_calcs
            .insert(variant.binding, calculation.clone());
        cmds.entity(entity).remove::<VariantShaderBuilder>();
    });
}

pub fn build_new_shaders(
    mut cmds: Commands,
    render_device: Res<RenderDevice>,
    mut query: Query<(Entity, &SdfPipelineKey, &mut SdfShaderBuilder)>,
    mut shaders: ResMut<SdfShaderRegister>,
    mut pipeline: ResMut<SdfPipeline>,
    assets: ResMut<AssetServer>,
) {
    for (entity, key, mut shader) in query.iter_mut() {
        if shaders.loaded_shaders.contains(key) {
            error!("Sdf Shader was already loaded for key '{key:?}'");
            continue;
        };

        let Some(entrys) =
            key.0
                .iter()
                .unique_by(|(_, b)| b)
                .try_fold(Vec::new(), |mut entrys, (_, binding)| {
                    let sdf_calc = shaders.loaded_calcs.get(binding)?;
                    entrys.push(sdf_calc.bindgroup_layout_entry());
                    shader.add_sdf_calculation(sdf_calc.clone());
                    Some(entrys)
                })
        else {
            error!("Not all sdf calculations found for shader with key '{key:?}'");
            continue;
        };

        shaders.loaded_shaders.insert(key.clone());
        let vertex_layout = shader.vertex_buffer_layout();
        let shader = shader.to_shader(key, &shaders.snippets);
        let shader = assets.add(shader);
        let bind_group_layout = render_device.create_bind_group_layout(None, &entrys);

        pipeline.specialization.insert(
            key.clone(),
            SdfSpecializationData {
                vertex_layout,
                bind_group_layout,
                shader,
            },
        );

        cmds.entity(entity).remove::<SdfShaderBuilder>();
    }
}

fn build_sdf_bindgroups(
    mut new_sdfs: EventReader<NewPipelineKey>,
    device: Res<RenderDevice>,
    mut pipeline: ResMut<SdfPipeline>,
) {
    for NewPipelineKey { key, .. } in new_sdfs.read() {
        let variant_entrys = key
            .0
            .iter()
            .map(|(_, i)| i)
            .unique()
            .map(|i| BindGroupEntry {
                binding: *i,
                resource: pipeline.bind_group_buffers[*i as usize]
                    .buffer()
                    .unwrap()
                    .as_entire_binding(),
            })
            .collect_vec();

        let Some(specialization) = pipeline.specialization.get(key) else {
            error!("No specialization entry for sdf pipeline key '{key:?}' found");
            continue;
        };

        let bind_group = device.create_bind_group(
            format!("Sdf {key:?}").as_str(),
            &specialization.bind_group_layout,
            &variant_entrys,
        );
        pipeline.bind_groups.insert(key.clone(), bind_group);
    }
}
