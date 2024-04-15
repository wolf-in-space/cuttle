use super::{
    pipeline::{SdfPipeline, SdfSpecializationData},
    shader::{
        buffers::{SdfOperationsBuffer, SdfRenderIndex, SdfVariantBuffer},
        loading::{LoadedSdfPipelineData, SdfShaderRegister},
    },
};
use crate::flag::{RenderSdf, RenderableVariant};
use bevy::{
    prelude::*,
    render::{
        render_resource::{BufferUsages, BufferVec},
        renderer::RenderDevice,
        Extract,
    },
};
use bevy_comdf_core::aabb::AABB;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct ExtractedSdfVariant {
    pub bytes: Vec<u8>,
    pub binding: u32,
    pub index: u32,
}

#[derive(Resource, Default, Debug)]
pub struct ExtractedSdfVariants {
    pub sdfs: Vec<ExtractedSdfVariant>,
}

pub fn extract_sdf_variants(
    mut extracted: ResMut<ExtractedSdfVariants>,
    query: Extract<Query<(&SdfVariantBuffer, &RenderableVariant, &SdfRenderIndex)>>,
) {
    extracted.sdfs.clear();
    extracted.sdfs.extend(
        query
            .iter()
            .map(|(bytes, variant, index)| ExtractedSdfVariant {
                bytes: bytes.clone().bytes(),
                binding: variant.binding,
                index: index.0,
            }),
    );
}

pub fn extract_render_sdfs(
    mut extracted: ResMut<ExtractedRenderSdfs>,
    query: Extract<Query<(&RenderSdf, &SdfOperationsBuffer, &AABB)>>,
) {
    extracted
        .0
        .extend(
            query
                .iter()
                .filter(|(sdf, _, _)| sdf.0.is_empty())
                .map(|(sdf, buffer, aabb)| ExtractedRenderSdf {
                    key: sdf.clone(),
                    bytes: buffer.0.clone(),
                    size: aabb.size(),
                    translation: aabb.pos(),
                }),
        );
}

pub struct ExtractedRenderSdf {
    pub key: RenderSdf,
    pub size: Vec2,
    pub translation: Vec2,
    pub bytes: Vec<u8>,
}

#[derive(Resource, Default)]
pub struct ExtractedRenderSdfs(pub Vec<ExtractedRenderSdf>);

pub fn extract_variant_data(
    register: Extract<Res<SdfShaderRegister>>,
    mut pipeline: ResMut<SdfPipeline>,
) {
    let variants = register.bindings.len();
    if pipeline.bind_group_buffers.len() < variants {
        pipeline
            .bind_group_buffers
            .resize_with(variants, || BufferVec::new(BufferUsages::STORAGE));
    }
}

pub fn extract_loaded_specialization_data(
    mut loaded: Extract<EventReader<LoadedSdfPipelineData>>,
    mut send: EventWriter<SdfSpecializationData>,
    render_device: Res<RenderDevice>,
) {
    for data in loaded.read() {
        let entrys = data
            .entrys
            .iter()
            .copied()
            .unique_by(|e| e.binding)
            .collect_vec();
        let bind_group_layout = render_device.create_bind_group_layout(None, &entrys);
        let variant = SdfSpecializationData {
            key: data.key.clone(),
            vertex_layout: data.vertex_layout.clone(),
            bind_group_layout,
            shader: data.shader.clone(),
        };
        send.send(variant);
    }
}
