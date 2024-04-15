use std::fmt::Debug;

use super::_shader::{SdfCalcBuilder, SdfShaderBuilder};
use crate::{
    flag::{SdfFlag, SdfRenderKey},
    operations::SdfRenderIndex,
};
use bevy::{
    core::{bytes_of, Pod},
    prelude::*,
    render::render_resource::BindGroupLayoutEntry,
    utils::{HashMap, HashSet},
};
use bevy_comdf_core::prepare::PrepareSdf;

pub fn plugin(app: &mut App) {
    app.add_event::<LoadedSdfPipelineData>();
}

#[derive(Clone, Component)]
pub struct SdfRenderBuffer {
    align: u8,
    bytes: Vec<u8>,
}

impl Default for SdfRenderBuffer {
    fn default() -> Self {
        Self {
            align: 0,
            bytes: Vec::new(),
        }
    }
}

impl SdfRenderBuffer {
    pub fn push<T: Pod + Debug>(&mut self, value: &T) {
        let bytes = bytes_of(value);
        let size = bytes.len() as u8;
        let align = if size == 12 { 16 } else { size };
        self.align = u8::max(self.align, align);
        self.pad_for_align(align);
        self.bytes.extend(bytes);
    }

    pub fn bytes(&mut self) -> Vec<u8> {
        self.pad_for_align(self.align);
        self.bytes.clone()
    }

    fn pad_for_align(&mut self, align: u8) {
        let len = self.bytes.len() as u8;
        let padding = len % align;
        if padding != 0 {
            let padding = align - padding;
            self.bytes.extend(0..padding);
        }
    }
}

pub fn clear_render_data(mut query: Query<&mut SdfRenderBuffer>) {
    query
        .iter_mut()
        .for_each(|mut render_data| render_data.bytes.clear())
}

#[derive(Component)]
pub struct LoadSdfRenderPipelineData;

#[derive(Resource, Default)]
pub struct SdfShaderRegister {
    pub bindings: HashMap<SdfFlag, u32>,
    loaded_calcs: HashMap<u32, SdfCalcBuilder>,
    shaders: HashSet<SdfRenderKey>,
}

pub fn start_calculation_build_if_not_registered(
    mut cmds: Commands,
    query: Query<(Entity, &SdfFlag), With<PrepareSdf>>,
    mut shaders: ResMut<SdfShaderRegister>,
) {
    query.iter().for_each(|(entity, flag)| {
        if !shaders.bindings.contains_key(flag) {
            let binding = shaders.bindings.len() as u32;
            cmds.entity(entity).insert(SdfCalcBuilder::new(binding));
            shaders.bindings.insert(*flag, binding);
        }
    })
}

pub fn gather_loaded_calculations(
    mut cmds: Commands,
    query: Query<(Entity, &SdfRenderIndex, &SdfCalcBuilder), With<PrepareSdf>>,
    mut shaders: ResMut<SdfShaderRegister>,
) {
    query.iter().for_each(|(entity, index, calculation)| {
        shaders
            .loaded_calcs
            .insert(index.binding, calculation.clone());
        cmds.entity(entity).remove::<SdfCalcBuilder>();
    });
}

#[derive(Debug, Event)]
pub struct LoadedSdfPipelineData {
    pub key: SdfRenderKey,
    pub shader: Handle<Shader>,
    pub entrys: Vec<BindGroupLayoutEntry>,
}

pub fn generate_and_send_shaders_for_new_keys(
    mut shaders: ResMut<SdfShaderRegister>,
    query: Query<(&SdfFlag, &SdfRenderKey, &SdfRenderIndex), With<PrepareSdf>>,
    mut events: EventWriter<LoadedSdfPipelineData>,
    mut shader_assets: ResMut<Assets<Shader>>,
) {
    query.iter().for_each(|(flag, key, index)| {
        if !shaders.shaders.contains(key) {
            let mut shader = SdfShaderBuilder::new(key.this);
            let mut entrys = Vec::with_capacity(key.operation_bindings.len());
            if let Some(()) = key
                .operation_bindings
                .iter()
                .chain(&[index.binding])
                .try_for_each(|target_flag| {
                    let sdf_calc = shaders.loaded_calcs.get(target_flag)?;
                    entrys.push(sdf_calc.bindgroup_layout_entry());
                    shader.add_sdf_calculation(sdf_calc.clone());
                    Some(())
                })
            {
                shaders.shaders.insert(key.clone());
                let shader = shader.to_shader(*flag, &shaders.bindings);
                let shader = shader_assets.add(shader);

                let event = LoadedSdfPipelineData {
                    key: key.clone(),
                    shader,
                    entrys,
                };
                events.send(event);
            }
        }
    });
}
