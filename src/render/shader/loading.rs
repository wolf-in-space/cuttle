use super::{building::SdfShaderBuilder, lines::Lines, variants::VariantShaderBuilder};
use crate::{flag::{RenderSdf, RenderableVariant, VariantFlag}, operations::OperationsFlag};
use bevy::{
    prelude::*,
    render::render_resource::{BindGroupLayoutEntry, VertexBufferLayout},
    utils::{HashMap, HashSet},
};
use std::fmt::Debug;

use crate::scheduling::ComdfRenderPostUpdateSet::*;
use crate::scheduling::ComdfRenderUpdateSet::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        assign_variant_bindings_and_start_shader_build_if_not_registered
            .in_set(AssignVariantBindings),
    );
    app.add_systems(
        PostUpdate,
        (
            start_sdf_shader_build_if_not_registered.before(BuildShaders),
            (
                gather_loaded_calculations,
                generate_and_send_shaders_for_new_keys,
            )
                .chain()
                .after(BuildShaders),
        ),
    );
    app.add_event::<LoadedSdfPipelineData>();
}

#[derive(Component)]
pub struct LoadSdfRenderPipelineData;

#[derive(Resource, Default)]
pub struct SdfShaderRegister {
    pub snippets: HashMap<OperationsFlag, Lines>,
    pub bindings: HashMap<VariantFlag, u32>,
    loaded_calcs: HashMap<u32, VariantShaderBuilder>,
    shaders: HashSet<RenderSdf>,
    loaded_shaders: HashSet<RenderSdf>,
}

pub fn assign_variant_bindings_and_start_shader_build_if_not_registered(
    mut cmds: Commands,
    mut query: Query<(Entity, &mut RenderableVariant)>,
    mut shaders: ResMut<SdfShaderRegister>,
) {
    query.iter_mut().for_each(|(entity, mut renderable)| {
        if let Some(binding) = shaders.bindings.get(&renderable.flag) {
            renderable.binding = *binding;
        } else {
            let binding = shaders.bindings.len() as u32;
            shaders.bindings.insert(renderable.flag, binding);

            renderable.binding = binding;
            cmds.entity(entity)
                .insert(VariantShaderBuilder::new(binding));
        }
    });
}

pub fn start_sdf_shader_build_if_not_registered(
    mut cmds: Commands,
    query: Query<(Entity, &RenderSdf)>,
    mut shaders: ResMut<SdfShaderRegister>,
) {
    query.iter().for_each(|(entity, renderable)| {
        if !shaders.shaders.contains(renderable) {
            shaders.shaders.insert(renderable.clone());
            cmds.entity(entity).insert(SdfShaderBuilder::new());
        }
    })
}

pub fn gather_loaded_calculations(
    mut cmds: Commands,
    query: Query<(Entity, &RenderableVariant, &VariantShaderBuilder)>,
    mut shaders: ResMut<SdfShaderRegister>,
) {
    query.iter().for_each(|(entity, variant, calculation)| {
        shaders
            .loaded_calcs
            .insert(variant.binding, calculation.clone());
        cmds.entity(entity).remove::<VariantShaderBuilder>();
    });
}

#[derive(Debug, Event)]
pub struct LoadedSdfPipelineData {
    pub key: RenderSdf,
    pub shader: Handle<Shader>,
    pub vertex_layout: VertexBufferLayout,
    pub entrys: Vec<BindGroupLayoutEntry>,
}

pub fn generate_and_send_shaders_for_new_keys(
    mut cmds: Commands,
    mut shaders: ResMut<SdfShaderRegister>,
    mut query: Query<(Entity, &RenderSdf, &mut SdfShaderBuilder)>,
    mut events: EventWriter<LoadedSdfPipelineData>,
    mut shader_assets: ResMut<Assets<Shader>>,
) {
    query.iter_mut().for_each(|(entity, key, mut shader)| {
        if !shaders.loaded_shaders.contains(key) {
            let mut entrys = Vec::with_capacity(key.0.len());
            if let Some(()) = key.0.iter().try_for_each(|(_, binding)| {
                let sdf_calc = shaders.loaded_calcs.get(binding)?;
                entrys.push(sdf_calc.bindgroup_layout_entry());
                shader.add_sdf_calculation(sdf_calc.clone());
                Some(())
            }) {
                shaders.loaded_shaders.insert(key.clone());
                let vertex_layout = shader.vertex_buffer_layout();
                let shader = shader.to_shader(key, &shaders.snippets);
                let shader = shader_assets.add(shader);

                let event = LoadedSdfPipelineData {
                    vertex_layout,
                    key: key.clone(),
                    shader,
                    entrys,
                };
                events.send(event);
                cmds.entity(entity).remove::<SdfShaderBuilder>();
            } else {
                warn!("Not all variant calculations found for shader with key '{key:?}'");
            }
        }
    });
}
