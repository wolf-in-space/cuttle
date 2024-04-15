use super::loading::SdfShaderRegister;
use crate::flag::RenderSdf;
use crate::flag::RenderableVariant;
use crate::scheduling::ComdfRenderPostUpdateSet::*;
use crate::scheduling::ComdfRenderUpdateSet::*;
use bevy::core::Pod;
use bevy::{core::bytes_of, prelude::*};
use bevy_comdf_core::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            add_a_if_with_b_and_without_a::<SdfRenderIndex, RenderableVariant>
                .before(AssignVariantIndices),
            assign_variant_indices.in_set(AssignVariantIndices),
        ),
    );
    app.add_systems(
        PostUpdate,
        (
            add_a_if_with_b_and_without_a::<SdfVariantBuffer, RenderableVariant>,
            add_a_if_with_b_and_without_a::<SdfOperationsBuffer, RenderSdf>,
            clear_variant_buffers,
            clear_operations_buffers,
        )
            .before(GatherDataForExtract),
    );
}

#[derive(Clone, Default, Component)]
pub struct SdfVariantBuffer {
    align: u8,
    bytes: Vec<u8>,
}

impl SdfVariantBuffer {
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

pub fn clear_variant_buffers(mut query: Query<&mut SdfVariantBuffer>) {
    query
        .iter_mut()
        .for_each(|mut render_data| render_data.bytes.clear())
}

#[derive(Clone, Default, Component)]
pub struct SdfOperationsBuffer(pub Vec<u8>);

pub fn clear_operations_buffers(mut query: Query<&mut SdfOperationsBuffer>) {
    query
        .iter_mut()
        .for_each(|mut render_data| render_data.0.clear())
}

#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord)]
pub struct SdfRenderIndex(pub u32);

pub fn assign_variant_indices(
    register: Res<SdfShaderRegister>,
    mut query: Query<(&mut SdfRenderIndex, &RenderableVariant)>,
) {
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
