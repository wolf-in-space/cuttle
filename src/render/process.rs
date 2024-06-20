use super::pipeline::SdfPipeline;
use super::pipeline::SdfPipelineKey;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::{
    render_resource::{BindGroup, BindGroupEntries, BufferUsages, BufferVec},
    renderer::{RenderDevice, RenderQueue},
    view::{ExtractedView, ViewUniforms},
};
use bevy_comdf_core::aabb::AABB;
use bytemuck::Pod;
use bytemuck::Zeroable;
use itertools::Itertools;

pub fn plugin(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);

    // render_app.add_systems(
    //     Render,
    //     (
    //         // batch_render_sdfs.in_set(PrepareBuffers),
    //         // (process_sdfs, prepare_view_bind_groups).in_set(PrepareBatches),
    //         // write_buffers.chain().in_set(WriteBuffers),
    //     ),
    // );
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SdfInstance {
    size: Vec2,
    position: Vec2,
    sdf_index: u32,
}

#[derive(Component)]
pub struct SdfBatch {
    pub instance_count: u16,
    pub key: SdfPipelineKey,
    pub vertex_buffer: BufferVec<SdfInstance>,
}
/*
pub fn batch_render_sdfs(
    mut cmds: Commands,
    render_sdfs: Query<(&SdfPipelineKey, &AABB, &SdfStorageIndex), With<RenderSdf>>,
) {
    let instances = render_sdfs
        .into_iter()
        .into_grouping_map_by(|(key, _, _)| *key)
        .fold(
            (Vec::new(), 0),
            |(mut vertices, count), _, (_, aabb, index)| {
                vertices.push(SdfInstance {
                    size: aabb.size(),
                    position: aabb.pos(),
                    sdf_index: index.0,
                });
                (vertices, count + 1)
            },
        )
        .into_iter()
        .fold(
            Vec::new(),
            |mut instances, (key, (vertices, instance_count))| {
                let mut vertex_buffer = BufferVec::new(BufferUsages::VERTEX);
                *vertex_buffer.values_mut() = vertices;
                instances.push(SdfBatch {
                    instance_count,
                    key: key.clone(),
                    vertex_buffer,
                });
                instances
            },
        );

    cmds.spawn_batch(instances);
}
pub fn process_sdfs(
    mut sdfs: Query<(&SdfBinding, &SdfStorageIndex, &mut SdfStorageBuffer)>,
    mut pipeline: ResMut<SdfPipeline>,
) {
    sdfs.iter_mut()
        .sorted_unstable_by_key(|(_, index, _)| **index)
        .for_each(|(SdfBinding(bind), _, mut buffer)| {
            pipeline.bind_group_buffers[*bind as usize].extend(buffer.bytes());
        });
}

pub fn write_buffers(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut pipeline: ResMut<SdfPipeline>,
    mut instances: Query<&mut SdfBatch>,
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

*/
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
