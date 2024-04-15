use super::extract::{ExtractedRenderSdf, ExtractedRenderSdfs};
use super::pipeline::SdfSpecializationData;
use super::{extract::ExtractedSdfVariants, pipeline::SdfPipeline};
use crate::flag::RenderSdf;
use bevy::core::bytes_of;
use bevy::render::render_resource::BindGroupEntry;
use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroup, BindGroupEntries, BufferUsages, BufferVec},
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, ViewUniforms},
    },
    utils::HashMap,
};
use itertools::Itertools;

#[derive(Resource, Default)]
pub struct SdfBindGroups(pub HashMap<RenderSdf, BindGroup>);

#[derive(Component)]
pub struct SdfInstance {
    pub key: RenderSdf,
    pub vertex_buffer: BufferVec<u8>,
}

pub fn process_render_sdfs(mut cmds: Commands, mut extracted: ResMut<ExtractedRenderSdfs>) {
    let instances = extracted
        .0
        .drain(..)
        .map(
            |ExtractedRenderSdf {
                 key,
                 size,
                 translation,
                 bytes,
             }| {
                let mut vertex_buffer = BufferVec::new(BufferUsages::VERTEX);
                vertex_buffer
                    .values_mut()
                    .extend_from_slice(bytes_of(&size));
                vertex_buffer
                    .values_mut()
                    .extend_from_slice(bytes_of(&translation));
                vertex_buffer.extend(bytes);

                SdfInstance { key, vertex_buffer }
            },
        )
        .collect_vec();

    cmds.spawn_batch(instances);
}

pub fn process_sdf_variants(
    mut extracted: ResMut<ExtractedSdfVariants>,
    mut pipeline: ResMut<SdfPipeline>,
) {
    extracted.sdfs.sort_unstable_by_key(|v| v.index);
    extracted.sdfs.drain(..).for_each(|v| {
        pipeline.bind_group_buffers[v.binding as usize].extend(v.bytes);
    });
}

pub fn create_bind_groups_for_new_keys(
    device: Res<RenderDevice>,
    mut recv: EventReader<SdfSpecializationData>,
    pipeline: Res<SdfPipeline>,
    mut bind_groups: ResMut<SdfBindGroups>,
) {
    recv.read().for_each(|data| {
        let variant_entrys = data
            .key
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

        bind_groups.0.insert(
            data.key.clone(),
            device.create_bind_group(None, &data.bind_group_layout, &variant_entrys),
        );
    });
}

pub fn write_buffers(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut pipeline: ResMut<SdfPipeline>,
    mut instances: Query<&mut SdfInstance>,
) {
    pipeline.indices.write_buffer(&device, &queue);
    pipeline.bind_group_buffers.iter_mut().for_each(|buffer| {
        buffer.write_buffer(&device, &queue);
        buffer.clear();
    });
    instances.iter_mut().for_each(|mut instance| {
        instance.vertex_buffer.write_buffer(&device, &queue);
    });
}

#[derive(Component)]
pub struct SdfViewBindGroup {
    pub value: BindGroup,
}

pub fn prepare_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<SdfPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in &views {
            let view_bind_group = render_device.create_bind_group(
                "sdf_view_bind_group",
                &pipeline.view_layout,
                &BindGroupEntries::single(view_binding.clone()),
            );

            commands.entity(entity).insert(SdfViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}
