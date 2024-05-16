use super::loading::SdfBinding;
use super::loading::SdfShaderRegister;
use crate::scheduling::ComdfRenderSet::*;
use bevy_app::prelude::*;
use bevy_core::bytes_of;
use bevy_core::Pod;
use bevy_ecs::prelude::*;
use bevy_render::Render;
use bevy_render::RenderApp;
use glam::Mat2;
use glam::Mat4;
use glam::Vec2;
use glam::Vec3;
use glam::Vec4;

pub fn plugin(app: &mut App) {
    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app.add_systems(Render, (assign_variant_indices.in_set(AssignIndices),));
}

#[derive(Clone, Default, Component)]
pub struct SdfStorageBuffer {
    max_align: u8,
    bytes: Vec<u8>,
}

impl SdfStorageBuffer {
    pub fn push<T: StorageBufferType>(&mut self, value: &T) {
        let bytes = bytes_of(value);
        debug_assert!(bytes.len() as u8 == T::size());
        self.max_align = u8::max(self.max_align, T::align());
        self.pad_for_align(T::align());
        self.bytes.extend(bytes);
    }

    pub fn bytes(&mut self) -> Vec<u8> {
        self.pad_for_align(self.max_align);
        self.bytes.clone()
    }

    fn pad_for_align(&mut self, align: u8) {
        let len = self.bytes.len() as u8;
        let remaining = len % align;
        if remaining != 0 {
            self.bytes.extend(0..(align - remaining));
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord)]
pub struct SdfStorageIndex(pub u32);

pub fn assign_variant_indices(
    register: Res<SdfShaderRegister>,
    mut query: Query<(&mut SdfStorageIndex, &SdfBinding)>,
) {
    let total_bindings = register.bindings.len();
    query.iter_mut().fold(
        vec![0; total_bindings],
        |mut binding_indices, (mut render_index, SdfBinding(bind))| {
            let index = &mut binding_indices[*bind as usize];
            render_index.0 = *index;
            *index += 1;
            binding_indices
        },
    );
}

pub trait StorageBufferType: Pod {
    fn align() -> u8;
    fn size() -> u8;
    fn wgsl_type_name() -> &'static str;
}

macro_rules! impl_storage_buf_type {
    ($type:ident, $align:expr, $size:expr, $name:expr) => {
        impl StorageBufferType for $type {
            fn align() -> u8 {
                $align
            }

            fn size() -> u8 {
                $size
            }

            fn wgsl_type_name() -> &'static str {
                $name
            }
        }
    };
}

impl_storage_buf_type!(f32, 4, 4, "f32");
impl_storage_buf_type!(u32, 4, 4, "u32");
impl_storage_buf_type!(i32, 4, 4, "i32");
impl_storage_buf_type!(Vec2, 8, 8, "vec2<f32>");
impl_storage_buf_type!(Vec3, 16, 12, "vec3<f32>");
impl_storage_buf_type!(Vec4, 16, 16, "vec4<f32>");
impl_storage_buf_type!(Mat2, 8, 16, "mat2x2<f32>");
impl_storage_buf_type!(Mat4, 16, 64, "mat4x4<f32>");
