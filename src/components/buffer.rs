use crate::{flag::CompFlag, shader::CompShaderInfos};
use bevy::{
    prelude::*,
    render::render_resource::{BufferUsages, RawBufferVec},
};
use bytemuck::{bytes_of, Pod, Zeroable};
use std::any::type_name;

pub struct SdfBuffer {
    pub buffer: RawBufferVec<u8>,
    pub stride: usize,
    pub current_index: usize,
}

impl SdfBuffer {
    pub fn new(stride: usize) -> Self {
        Self {
            buffer: RawBufferVec::new(BufferUsages::STORAGE),
            stride,
            current_index: 0,
        }
    }

    pub fn prep_for_push(&mut self, index: usize, comp_offset: usize) {
        self.current_index = index * self.stride + comp_offset;
    }

    pub fn push<T: BufferType>(&mut self, value: &T) {
        let bytes = bytes_of(value);
        let Some(slice) = self
            .buffer
            .values_mut()
            .get_mut(self.current_index..(self.current_index + bytes.len()))
        else {
            error_once!(
                "Pushing to Buffer out of range: type = {}, start = {}, amount = {}, stride={}, buffer_len = {}",
                type_name::<T>(), self.current_index, bytes.len(), self.stride, self.buffer.len()
            );
            return;
        };
        slice.copy_from_slice(bytes);
        self.current_index += bytes.len();
    }

    pub fn stride_and_offsets_for_flag(
        flag: &CompFlag,
        infos: &CompShaderInfos,
    ) -> (usize, Vec<(u8, usize)>) {
        let mut offsets = Vec::default();
        let mut max_align = 0;
        let mut offset = 0;

        let align_offset = |offset: &mut usize, align: usize| {
            let remaining = *offset % align;
            // println!(
            //     "offset={offset}, align={align}, remaining={remaining}, paddings={}",
            //     align - remaining
            // );
            if remaining != 0 {
                *offset += align - remaining
            }
        };

        for index in flag.ones() {
            let info = &infos[index];
            for input in info.inputs.iter() {
                let align = input.type_info.align as usize;
                max_align = usize::max(max_align, align);
                align_offset(&mut offset, align);
            }

            offsets.push((index as u8, offset));

            for input in info.inputs.iter() {
                offset += input.type_info.size as usize;
            }
        }

        if max_align > 0 {
            align_offset(&mut offset, max_align);
        }

        (offset, offsets)
    }
}

pub trait BufferType: Pod {
    fn descriptor() -> BufferTypeInfo;
    fn shader_input(name: &'static str) -> ShaderInput {
        ShaderInput {
            type_info: Self::descriptor(),
            name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferTypeInfo {
    pub align: u8,
    pub size: u8,
    pub wgsl_type: &'static str,
}

impl Default for BufferTypeInfo {
    fn default() -> Self {
        Self {
            align: u8::MAX,
            size: u8::MAX,
            wgsl_type: "UNINITIALIZED",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderInput {
    pub type_info: BufferTypeInfo,
    pub name: &'static str,
}

#[macro_export]
macro_rules! impl_storage_buf_type {
    ($type:ident, $align:expr, $size:expr, $name:expr) => {
        impl BufferType for $type {
            fn descriptor() -> BufferTypeInfo {
                BufferTypeInfo {
                    align: $align,
                    size: $size,
                    wgsl_type: $name,
                }
            }
        }
    };
}

#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct SdfResult;

impl_storage_buf_type!(f32, 4, 4, "f32");
impl_storage_buf_type!(u32, 4, 4, "u32");
impl_storage_buf_type!(i32, 4, 4, "i32");
impl_storage_buf_type!(Vec2, 8, 8, "vec2<f32>");
impl_storage_buf_type!(Vec3, 16, 12, "vec3<f32>");
impl_storage_buf_type!(Vec4, 16, 16, "vec4<f32>");
impl_storage_buf_type!(Mat2, 8, 16, "mat2x2<f32>");
impl_storage_buf_type!(Mat4, 16, 64, "mat4x4<f32>");
impl_storage_buf_type!(SdfResult, 4, 8, "SdfResult");

#[cfg(test)]
mod tests {
    use crate::{
        components::{buffer::SdfBuffer, RenderSdfComponent},
        flag::CompFlag,
        prelude::Fill,
        shader::CompShaderInfos,
    };
    use bevy::transform::components::GlobalTransform;
    use bevy_comdf_core::components::*;
    use fixedbitset::FixedBitSet;

    fn prep_buffer_infos() -> CompShaderInfos {
        let mut infos = CompShaderInfos::default();
        infos.register(Point::shader_info());
        infos.register(Line::shader_info());
        infos.register(Rectangle::shader_info());
        infos.register(Fill::shader_info());
        infos.register(Added::shader_info());
        infos.register(Rotated::shader_info());
        infos.register(GlobalTransform::shader_info());
        infos
    }

    fn test(flag: usize, expected: (usize, Vec<(u8, usize)>)) {
        let flag = CompFlag(FixedBitSet::with_capacity_and_blocks(64, [flag]));
        let infos = prep_buffer_infos();
        assert_eq!(
            SdfBuffer::stride_and_offsets_for_flag(&flag, &infos),
            expected
        );
    }

    #[test]
    fn stride_and_offset() {
        test(0, (0, vec![]));
        test(1, (0, vec![(0, 0)]));
        test(0b10, (4, vec![(1, 0)]));
        test(0b11, (4, vec![(0, 0), (1, 0)]));
        test(0b111, (16, vec![(0, 0), (1, 0), (2, 8)]));
        test(
            0b111111,
            (48, vec![(0, 0), (1, 0), (2, 8), (3, 16), (4, 28), (5, 32)]),
        );
        test(0b1010, (32, vec![(1, 0), (3, 16)]));
        test(0b1001010, (96, vec![(1, 0), (3, 16), (6, 32)]));
        test(0b1001010, (96, vec![(1, 0), (3, 16), (6, 32)]));
    }
}
