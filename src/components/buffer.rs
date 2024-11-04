use super::{SdfCompInfos, SdfRenderData};
use crate::pipeline::{specialization::SdfPipeline, ComdfRenderSet};
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
            .init_resource::<CompBufferBindgroup>()
            .add_systems(
                Render,
                (
                    write_comp_buffers.ambiguous_with_all(),
                    build_buffer_bindgroup,
                )
                    .chain()
                    .in_set(ComdfRenderSet::Buffer),
            );
    }

    fn finish(&self, app: &mut App) {
        let infos = app.world().resource::<SdfCompInfos>();
        let (insert_fns, debug_fns, write_fns, get_fns) = process_sdf_infos(infos);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(DebugBufferFns(debug_fns));
        render_app.insert_resource(WriteBufferFns(write_fns));
        render_app.insert_resource(GetBufferBindingResFns(get_fns));

        let render_world = render_app.world_mut();
        let mut buffer_entity = render_world.spawn(BufferEntity);
        for func in insert_fns {
            func(&mut buffer_entity);
        }
    }
}

fn process_sdf_infos(
    infos: &SdfCompInfos,
) -> (
    Vec<InsertBufferFn>,
    Vec<DebugBufferFn>,
    Vec<WriteBufferFn>,
    Vec<GetBufferBindingResFn>,
) {
    (
        infos.iter().map(|i| i.buffer.insert).collect(),
        infos.iter().map(|i| i.buffer.debug).collect(),
        infos.iter().map(|i| i.buffer.write).collect(),
        infos.iter().map(|i| i.buffer.get).collect(),
    )
}

#[derive(Debug)]
pub struct BufferInfo {
    insert: InsertBufferFn,
    debug: DebugBufferFn,
    write: WriteBufferFn,
    get: GetBufferBindingResFn,
}

impl BufferInfo {
    pub(crate) fn new<C: SdfRenderData>() -> Self {
        Self {
            insert: CompBuffer::<C>::insert,
            debug: CompBuffer::<C>::debug,
            write: CompBuffer::<C>::write,
            get: CompBuffer::<C>::get_binding_res,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct CompBuffer<C: SdfRenderData>(StorageBuffer<Vec<C>>);

pub type InsertBufferFn = fn(&mut EntityWorldMut);
pub type DebugBufferFn = fn(&EntityRef, u32);
pub type WriteBufferFn = fn(&mut EntityMut, &RenderDevice, &RenderQueue);
pub type GetBufferBindingResFn = for<'a> fn(&'a EntityRef<'a>) -> BindingResource<'a>;

impl<C: SdfRenderData> CompBuffer<C> {
    pub fn insert(entity: &mut EntityWorldMut) {
        entity.insert(Self::default());
    }

    pub fn debug(entity: &EntityRef, index: u32) {
        let buf = entity.get::<Self>().unwrap().get();
        println!("{}/{}: {:?}", index, buf.len(), buf.get(index as usize));
    }

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

impl<C: SdfRenderData> Default for CompBuffer<C> {
    fn default() -> Self {
        Self(default())
    }
}

#[derive(Component)]
pub(crate) struct BufferEntity;

#[derive(Resource, Deref)]
pub(crate) struct WriteBufferFns(Vec<WriteBufferFn>);

fn write_comp_buffers(
    mut entity: Single<EntityMut, With<BufferEntity>>,
    write_fns: Res<WriteBufferFns>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    for func in write_fns.iter() {
        func(&mut entity, &device, &queue);
    }
}

#[derive(Resource, Deref, Default)]
pub struct CompBufferBindgroup(pub Option<BindGroup>);

#[derive(Resource, Deref)]
pub(crate) struct GetBufferBindingResFns(Vec<GetBufferBindingResFn>);

fn build_buffer_bindgroup(
    entity: Single<EntityRef, With<BufferEntity>>,
    binding_res_fns: Res<GetBufferBindingResFns>,
    device: Res<RenderDevice>,
    pipeline: Res<SdfPipeline>,
    mut bindgroup: ResMut<CompBufferBindgroup>,
) {
    let bindign_res: Vec<BindGroupEntry> = binding_res_fns
        .iter()
        .enumerate()
        .map(|(i, func)| BindGroupEntry {
            binding: i as u32,
            resource: func(&entity),
        })
        .collect();
    bindgroup.0 = Some(device.create_bind_group(
        "sdf component buffers",
        &pipeline.comp_layout,
        &bindign_res,
    ));
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

#[derive(Resource, Deref)]
pub(crate) struct DebugBufferFns(Vec<DebugBufferFn>);
