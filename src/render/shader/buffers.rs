use super::loading::SdfShaderRegister;
use crate::flag::RenderableSdf;
use crate::scheduling::ComdfRenderSet::*;
use bevy_app::prelude::*;
use bevy_core::bytes_of;
use bevy_core::Pod;
use bevy_ecs::prelude::*;
use bevy_render::Render;
use bevy_render::RenderApp;

pub fn plugin(app: &mut App) {
    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app.add_systems(Render, (assign_variant_indices.in_set(AssignSdfIndices),));
}

#[derive(Clone, Default, Component)]
pub struct SdfStorageBuffer {
    align: u8,
    bytes: Vec<u8>,
}

impl SdfStorageBuffer {
    pub fn push<T: Pod>(&mut self, value: &T) {
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
            self.bytes.extend(0..(align - padding));
        }
    }
}

#[derive(Clone, Default, Component)]
pub struct SdfOperationsBuffer(pub Vec<u8>);
/*
pub fn clear_variant_buffers(mut query: Query<&mut SdfStorageBuffer>) {
    query
        .iter_mut()
        .for_each(|mut render_data| render_data.bytes.clear())
}

pub fn clear_operations_buffers(mut query: Query<&mut SdfOperationsBuffer>) {
    query
        .iter_mut()
        .for_each(|mut render_data| render_data.0.clear())
}
 */
#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord)]
pub struct SdfStorageIndex(pub u32);

pub fn assign_variant_indices(
    register: Res<SdfShaderRegister>,
    mut query: Query<(&mut SdfStorageIndex, &RenderableSdf)>,
) {
    // println!("assign_indices: {}", query.iter().len());
    let total_bindings = register.bindings.len();
    query.iter_mut().fold(
        vec![0; total_bindings],
        |mut binding_indices, (mut render_index, variant)| {
            let index = &mut binding_indices[variant.binding as usize];
            render_index.0 = *index;
            *index += 1;
            binding_indices
        },
    );
}
