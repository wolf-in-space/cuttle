use crate::components::initialization::CuttleRenderData;
use crate::groups::{ConfigId, CuttleConfig};
use crate::pipeline::extract::Extracted;
use crate::pipeline::CuttleRenderSet;
use bevy::ecs::world::{EntityMutExcept, EntityRefExcept};
use bevy::render::render_resource::encase::private::WriteInto;
use bevy::render::render_resource::ShaderType;
use bevy::utils::HashMap;
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
use std::marker::PhantomData;

pub struct BufferPlugin;
impl Plugin for BufferPlugin {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).add_systems(
            Render,
            (
                write_comp_buffers.ambiguous_with_all(),
                build_buffer_bind_groups,
            )
                .chain()
                .in_set(CuttleRenderSet::ComponentBuffers),
        );
    }
}

#[derive(Component)]
pub struct CuttleBuffer<Comp, Render, Storage: ShaderType> {
    storage: StorageBuffer<Storage>,
    to_render_data: fn(&Comp) -> Render,
}

pub type CompBuffer<C, R> = CuttleBuffer<C, R, Vec<R>>;
impl<Comp, Render> CompBuffer<Comp, Render>
where
    Comp: Component,
    Render: CuttleRenderData,
{
    pub fn insert(&mut self, index: usize, comp: &Comp) {
        let value = (self.to_render_data)(comp);
        *self.storage.get_mut().get_mut(index).unwrap() = value;
    }

    pub fn resize(&mut self, size: usize) {
        let buffer = self.storage.get_mut();
        buffer.resize_with(size, || Render::default());
    }
}

pub type GlobalBuffer<C, R> = CuttleBuffer<C, R, R>;
impl<Comp, Render> GlobalBuffer<Comp, Render>
where
    Comp: Component,
    Render: CuttleRenderData,
{
    pub fn set(&mut self, comp: &Comp) {
        let value = (self.to_render_data)(comp);
        *self.storage.get_mut() = value;
    }
}

impl<Comp, Render, Storage> CuttleBuffer<Comp, Render, Storage>
where
    Comp: Component,
    Render: CuttleRenderData,
    Storage: ShaderType + Default + WriteInto + Send + Sync + 'static,
{
    pub fn new(to_render_data: fn(&Comp) -> Render) -> Self {
        Self {
            storage: StorageBuffer::default(),
            to_render_data,
        }
    }

    pub fn init(
        app: &mut App,
        buffer_entity: Entity,
        to_render_data: fn(&Comp) -> Render,
    ) -> usize {
        let render_world = app.sub_app_mut(RenderApp).world_mut();
        let mut ent = render_world.entity_mut(buffer_entity);
        let mut buffer_fns = ent.get_mut::<BufferFns>().unwrap();

        let count = buffer_fns.bindings.len();
        buffer_fns.write.push(Self::write);
        buffer_fns.bindings.push(Self::get_binding_res);
        ent.insert(Self::new(to_render_data));
        count
    }

    pub fn write(entity: &mut EntMut, device: &RenderDevice, queue: &RenderQueue) {
        if let Some(mut buffer) = entity.get_mut::<Self>() {
            buffer.storage.write_buffer(device, queue);
        }
    }

    pub fn get_binding_res<'a>(entity: &'a EntRef<'a>) -> BindingResource<'a> {
        entity
            .get::<Self>()
            .unwrap()
            .storage
            .buffer()
            .unwrap()
            .as_entire_binding()
    }
}

type EntMut<'w> = EntityMutExcept<'w, BufferFns>;
pub type WriteBufferFn = fn(&mut EntMut, &RenderDevice, &RenderQueue);
type EntRef<'w> = EntityRefExcept<'w, (Bind, BufferFns, BindLayout)>;
pub type GetBufferBindingResFn = for<'a> fn(&'a EntRef<'a>) -> BindingResource<'a>;

#[derive(Component, Default)]
pub struct BufferFns {
    pub write: Vec<WriteBufferFn>,
    pub bindings: Vec<GetBufferBindingResFn>,
}

#[derive(Component)]
#[require(Bind, BufferFns)]
pub struct CompBufferEntity;

#[derive(Component)]
#[require(Bind, BufferFns, Extracted)]
pub struct ConfigRenderEntity<G: CuttleConfig>(PhantomData<G>);

impl<G: CuttleConfig> ConfigRenderEntity<G> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

fn write_comp_buffers(
    mut buffers: Query<(EntityMutExcept<BufferFns>, &BufferFns)>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    for (mut ent_mut, buffer_fns) in &mut buffers {
        for write_fn in buffer_fns.write.iter() {
            write_fn(&mut ent_mut, &device, &queue);
        }
    }
}

#[derive(Component, Deref, Default)]
pub struct Bind(pub Option<BindGroup>);

#[derive(Component, Deref)]
pub struct BindLayout(pub BindGroupLayout);

pub fn build_comp_layout(
    mut cmds: Commands,
    buffer_entity: Single<(Entity, &BufferFns), With<CompBufferEntity>>,
    device: Res<RenderDevice>,
) -> BindGroupLayout {
    let (entity, fns) = *buffer_entity;
    let layout = build_buffer_layout(
        fns.bindings.len() as u32,
        &device,
        "cuttle components bind group layout",
    );
    cmds.entity(entity).insert(BindLayout(layout.clone()));
    layout
}

pub fn build_global_layouts(
    mut cmds: Commands,
    query: Query<(Entity, &ConfigId, &BufferFns)>,
    device: Res<RenderDevice>,
) -> HashMap<ConfigId, BindGroupLayout> {
    let mut result = HashMap::new();

    for (entity, id, fns) in &query {
        let name = format!("cuttle global bind group layouts for {id:?}");
        let layout = build_buffer_layout(fns.bindings.len() as u32, &device, name.as_str());
        cmds.entity(entity).insert(BindLayout(layout.clone()));
        result.insert(*id, layout);
    }

    result
}

fn build_buffer_bind_groups(
    mut query: Query<(EntRef, &BufferFns, &mut Bind, &BindLayout)>,
    device: Res<RenderDevice>,
) {
    for (entity, fns, mut bind, layout) in &mut query {
        let entries: Vec<BindGroupEntry> = fns
            .bindings
            .iter()
            .enumerate()
            .map(|(i, func)| BindGroupEntry {
                binding: i as u32,
                resource: func(&entity),
            })
            .collect();

        bind.0 = Some(device.create_bind_group("cuttle component buffers", &layout, &entries));
    }
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
