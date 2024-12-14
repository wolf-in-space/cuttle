use crate::components::initialization::CuttleRenderData;
use crate::pipeline::{specialization::CuttlePipeline, CuttleRenderSet};
use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource,
            BindingType, BufferBindingType, ShaderStages, StorageBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        Render, RenderApp,
    },
};

pub struct BufferPlugin;
impl Plugin for BufferPlugin {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<BufferFns>()
            .init_resource::<CompBufferBindgroup>()
            .add_systems(
                Render,
                (
                    write_comp_buffers.ambiguous_with_all(),
                    build_buffer_bindgroup,
                )
                    .chain()
                    .in_set(CuttleRenderSet::Buffer),
            );
    }
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct CompBuffer<C: CuttleRenderData>(StorageBuffer<Vec<C>>);

impl<C: CuttleRenderData> Default for CompBuffer<C> {
    fn default() -> Self {
        Self(default())
    }
}

impl<C: CuttleRenderData> CompBuffer<C> {
    pub fn write(entity: &mut EntityMut, device: &RenderDevice, queue: &RenderQueue) {
        if let Some(mut buffer) = entity.get_mut::<Self>() {
            buffer.write_buffer(device, queue);
        }
    }

    pub fn get_binding_res<'a>(entity: &'a EntityRef<'a>) -> BindingResource<'a> {
        entity
            .get::<Self>()
            .unwrap()
            .buffer()
            .unwrap()
            .as_entire_binding()
    }
}

pub type WriteBufferFn = fn(&mut EntityMut, &RenderDevice, &RenderQueue);
pub type GetBufferBindingResFn = for<'a> fn(&'a EntityRef<'a>) -> BindingResource<'a>;

#[derive(Resource, Default)]
pub(crate) struct BufferFns {
    pub write: Vec<WriteBufferFn>,
    pub bindings: Vec<GetBufferBindingResFn>,
}

#[derive(Component)]
pub(crate) struct BufferEntity;

fn write_comp_buffers(
    mut entity: Single<EntityMut, With<BufferEntity>>,
    fns: Res<BufferFns>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    for write_fn in fns.write.iter() {
        write_fn(&mut entity, &device, &queue);
    }
}

#[derive(Resource, Deref, Default)]
pub struct CompBufferBindgroup(pub Option<BindGroup>);

fn build_buffer_bindgroup(
    entity: Single<EntityRef, With<BufferEntity>>,
    fns: Res<BufferFns>,
    device: Res<RenderDevice>,
    pipeline: Res<CuttlePipeline>,
    mut bindgroup: ResMut<CompBufferBindgroup>,
) {
    let entries: Vec<BindGroupEntry> = fns
        .bindings
        .iter()
        .enumerate()
        .map(|(i, func)| BindGroupEntry {
            binding: i as u32,
            resource: func(&entity),
        })
        .collect();
    bindgroup.0 =
        Some(device.create_bind_group("sdf component buffers", &pipeline.comp_layout, &entries));
}

pub fn build_buffer_layout(count: u32, device: &RenderDevice, name: &str) -> BindGroupLayout {
    let entries: Vec<BindGroupLayoutEntry> = (0..count)
        .map(|binding| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::FRAGMENT,
            count: None,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        })
        .collect();
    device.create_bind_group_layout(name, &entries)
}
