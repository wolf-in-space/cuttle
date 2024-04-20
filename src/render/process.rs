use super::pipeline::SdfPipeline;
use super::shader::buffers::SdfOperationsBuffer;
use super::shader::buffers::SdfStorageBuffer;
use crate::flag::RenderableSdf;
use crate::flag::SdfPipelineKey;
use crate::prelude::SdfStorageIndex;
use crate::scheduling::ComdfRenderSet::*;
use bevy_app::App;
use bevy_comdf_core::aabb::AABB;
use bevy_core::bytes_of;
use bevy_ecs::prelude::*;
use bevy_render::Render;
use bevy_render::RenderApp;
use bevy_render::{
    render_resource::{BindGroup, BindGroupEntries, BufferUsages, BufferVec},
    renderer::{RenderDevice, RenderQueue},
    view::{ExtractedView, ViewUniforms},
};
use itertools::Itertools;

pub fn plugin(app: &mut App) {
    let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app.add_systems(
        Render,
        (
            process_render_sdfs.in_set(PrepareBuffers),
            (process_sdfs, prepare_view_bind_groups).in_set(PrepareBatches),
            write_buffers.chain().in_set(WriteBuffers),
        ),
    );
}

#[derive(Component)]
pub struct SdfInstance {
    pub key: SdfPipelineKey,
    pub vertex_buffer: BufferVec<u8>,
}

pub fn process_render_sdfs(
    mut cmds: Commands,
    render_sdfs: Query<(Entity, &SdfPipelineKey, &AABB, &SdfOperationsBuffer)>,
) {
    // println!("process_render_sdfs: {}", render_sdfs.iter().len());
    for (entity, key, aabb, buffer) in render_sdfs.into_iter() {
        let mut vertex_buffer = BufferVec::new(BufferUsages::VERTEX);
        vertex_buffer
            .values_mut()
            .extend_from_slice(bytes_of(&aabb.size()));
        vertex_buffer
            .values_mut()
            .extend_from_slice(bytes_of(&aabb.pos()));
        vertex_buffer.extend(buffer.0.clone());

        cmds.entity(entity).insert(SdfInstance {
            key: key.clone(),
            vertex_buffer,
        });
    }
}

pub fn process_sdfs(
    mut sdfs: Query<(&RenderableSdf, &SdfStorageIndex, &mut SdfStorageBuffer)>,
    mut pipeline: ResMut<SdfPipeline>,
) {
    // println!("process_sdfs: {}", sdfs.iter().len());
    sdfs.iter_mut()
        .sorted_unstable_by_key(|(_, index, _)| **index)
        .for_each(|(RenderableSdf { binding, .. }, _, mut buffer)| {
            pipeline.bind_group_buffers[*binding as usize].extend(buffer.bytes());
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
        // println!("buffer {:?}", buffer.values());
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
