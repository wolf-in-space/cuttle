use super::{building::SdfShaderBuilder, lines::Lines, variants::SdfCalculationBuilder};
use crate::render::pipeline::{SdfPipeline, SdfSpecializationData};
use crate::scheduling::ComdfRenderSet::*;
use crate::RenderSdf;
use crate::{flag::SdfPipelineKey, operations::OperationsFlag};
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

    render_app.add_event::<NewPipeline>();
    render_app.add_event::<NewRenderableSdfType>();
    render_app.add_systems(
        Render,
        (
            assign_bindings_and_trigger_events_for_new_keys.in_set(AssignBindings),
            create_new_bind_group_buffers
                .after(AssignBindings)
                .before(PrepareBatches),
            prepare_sdf_shader_build.in_set(PrepareShaderBuild),
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
    pub bindings: HashMap<SdfPipelineKey, u32>,
    loaded_calcs: HashMap<u32, SdfCalculationBuilder>,
    pipelines: HashSet<SdfPipelineKey>,
}

#[derive(Event, Debug)]
pub struct NewPipeline {
    entity: Entity,
    key: SdfPipelineKey,
}

#[derive(Event, Debug)]
pub struct NewRenderableSdfType {
    entity: Entity,
    binding: u32,
}

#[derive(Component, Clone, Copy)]
pub struct SdfBinding(pub u32);

#[derive(Component)]
pub struct SdfBindings(pub Vec<u32>);

pub fn assign_bindings_and_trigger_events_for_new_keys(
    mut cmds: Commands,
    mut query: Query<(Entity, &SdfPipelineKey, Option<&RenderSdf>)>,
    mut registry: ResMut<SdfShaderRegister>,
    mut new_pipeline: EventWriter<NewPipeline>,
    mut new_renderable: EventWriter<NewRenderableSdfType>,
) {
    query.iter_mut().for_each(|(entity, key, render)| {
        if render.is_some() && !registry.pipelines.contains(key) {
            registry.pipelines.insert(key.clone());
            new_pipeline.send(NewPipeline {
                entity,
                key: key.clone(),
            });
        }

        let mut insert = |bind: u32| {
            cmds.entity(entity)
                .insert((SdfBinding(bind), SdfBindings(vec![bind])));
        };

        if let Some(binding) = registry.bindings.get(key) {
            insert(*binding);
        } else {
            let binding = registry.bindings.len() as u32;
            registry.bindings.insert(key.clone(), binding);
            insert(binding);
            new_renderable.send(NewRenderableSdfType { entity, binding });
        }
    });
}

pub fn prepare_sdf_shader_build(
    mut cmds: Commands,
    mut new_keys: EventReader<NewRenderableSdfType>,
) {
    new_keys.read().for_each(|new| {
        cmds.entity(new.entity)
            .insert(SdfCalculationBuilder::new(new.binding));
    })
}

pub fn create_new_bind_group_buffers(
    mut pipeline: ResMut<SdfPipeline>,
    mut new_sdfs: EventReader<NewRenderableSdfType>,
) {
    new_sdfs.read().for_each(|_| {
        let buffer = BufferVec::new(BufferUsages::STORAGE);
        pipeline.bind_group_buffers.push(buffer);
    })
}

pub fn gather_loaded_calculations(
    mut cmds: Commands,
    query: Query<(Entity, &SdfBinding, &SdfCalculationBuilder)>,
    mut shaders: ResMut<SdfShaderRegister>,
) {
    query
        .iter()
        .for_each(|(entity, SdfBinding(bind), calculation)| {
            shaders.loaded_calcs.insert(*bind, calculation.clone());
            cmds.entity(entity).remove::<SdfCalculationBuilder>();
        });
}

pub fn build_new_shaders(
    mut cmds: Commands,
    render_device: Res<RenderDevice>,
    query: Query<(Entity, &SdfPipelineKey, &SdfBinding, &SdfBindings), With<RenderSdf>>,
    mut new_keys: EventReader<NewPipeline>,
    shaders: ResMut<SdfShaderRegister>,
    mut pipeline: ResMut<SdfPipeline>,
    assets: ResMut<AssetServer>,
) {
    for new_key in new_keys.read() {
        let Ok((entity, key, SdfBinding(bind), bindings)) = query.get(new_key.entity) else {
            error!("Entity {:?}, which was the first with key {:?} disappeared before the pipeline could be built", new_key.key, new_key.entity);
            continue;
        };

        let mut shader_builder = SdfShaderBuilder::new(*bind);

        let Some(entrys) =
            bindings
                .0
                .iter()
                .unique()
                .try_fold(Vec::new(), |mut entrys, binding| {
                    let sdf_calc = shaders.loaded_calcs.get(binding)?;
                    entrys.push(sdf_calc.bindgroup_layout_entry());
                    shader_builder.add_sdf_calculation(sdf_calc.clone());
                    Some(entrys)
                })
        else {
            error!("Not all sdf calculations found for shader with key '{key:?}'");
            continue;
        };

        let vertex_layout = shader_builder.vertex_buffer_layout();
        let shader = shader_builder.to_shader(key, &shaders.snippets);
        let shader = assets.add(shader);
        let bind_group_layout =
            render_device.create_bind_group_layout(format!("{:?}", key).as_str(), &entrys);

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
    bindings: Query<&SdfBindings, With<RenderSdf>>,
    mut new_sdfs: EventReader<NewPipeline>,
    device: Res<RenderDevice>,
    mut pipeline: ResMut<SdfPipeline>,
) {
    let mut build_bindgroup = |NewPipeline { entity, key }: &NewPipeline| -> Result<(), &str> {
        let variant_entrys = bindings
            .get(*entity)
            .map_err(|_| "Entity no longer exists or has no SdfBindings and RenderSdf components")?
            .0
            .iter()
            .unique()
            .map(|i| {
                Ok(BindGroupEntry {
                    binding: *i,
                    resource: pipeline
                        .bind_group_buffers
                        .get(*i as usize)
                        .ok_or("binding out of bounds for pipeline.bind_group_buffers")?
                        .buffer()
                        .ok_or("Bindgroup buffer was not written to")?
                        .as_entire_binding(),
                })
            })
            .collect::<Result<Vec<_>, &str>>()?;

        let specialization = pipeline
            .specialization
            .get(key)
            .ok_or("No specialization entry found")?;
        let bind_group = device.create_bind_group(
            format!("Sdf {key:?}").as_str(),
            &specialization.bind_group_layout,
            &variant_entrys,
        );
        pipeline.bind_groups.insert(key.clone(), bind_group);

        Ok(())
    };

    for new_pipeline @ NewPipeline { entity, key } in new_sdfs.read() {
        if let Err(err) = build_bindgroup(new_pipeline) {
            error!(
                "Failed to build bindgroup for Entity '{:?}' with sdf key: '{:?}' due to: '{}'",
                entity, key, err
            )
        }
    }
}
