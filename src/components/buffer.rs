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
use std::any::type_name;

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
                    .in_set(CuttleRenderSet::ComponentBuffers),
            );
    }
}

#[derive(Component)]
pub(crate) struct CompBuffer<C: Component, R: CuttleRenderData> {
    storage: StorageBuffer<Vec<R>>,
    to_render_data: fn(&C) -> R,
}

impl<C: Component, R: CuttleRenderData> CompBuffer<C, R> {
    pub fn new(to_render_data: fn(&C) -> R) -> Self {
        Self {
            storage: StorageBuffer::default(),
            to_render_data,
        }
    }

    pub fn set(&mut self, index: usize, comp: &C) {
        let value = (self.to_render_data)(comp);
        trace!(
            "EXTRACT_COMP_VAL: Comp={}, RenderType={}, val={:?}",
            type_name::<C>(),
            type_name::<R>(),
            value
        );
        *self.storage.get_mut().get_mut(index).unwrap() = value;
    }

    pub fn resize(&mut self, size: usize) {
        let buffer = self.storage.get_mut();
        buffer.resize_with(size, || R::default());
    }

    pub fn init(app: &mut App, buffer_entity: Entity, to_render_data: fn(&C) -> R) {
        let render_world = app.sub_app_mut(RenderApp).world_mut();

        render_world
            .entity_mut(buffer_entity)
            .insert(CompBuffer::<C, R>::new(to_render_data));

        let mut buffer_fns = render_world.resource_mut::<BufferFns>();
        buffer_fns.write.push(CompBuffer::<C, R>::write);
        buffer_fns
            .bindings
            .push(CompBuffer::<C, R>::get_binding_res);
    }

    pub fn write(entity: &mut EntityMut, device: &RenderDevice, queue: &RenderQueue) {
        if let Some(mut buffer) = entity.get_mut::<Self>() {
            buffer.storage.write_buffer(device, queue);
        }
    }

    pub fn get_binding_res<'a>(entity: &'a EntityRef<'a>) -> BindingResource<'a> {
        entity
            .get::<Self>()
            .unwrap()
            .storage
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
        Some(device.create_bind_group("cuttle component buffers", &pipeline.comp_layout, &entries));
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
