use crate::pipeline::extract::{ExtractedCuttleFlags, RenderIndexRange};
use crate::pipeline::{specialization::CuttlePipeline, CuttleRenderSet};
use bevy::ecs::component::{ComponentHooks, StorageType};
use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroup, BindGroupEntries, StorageBuffer},
        renderer::{RenderDevice, RenderQueue},
        Render, RenderApp,
    },
};
use std::fmt::Debug;

pub fn plugin(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<CompIndicesBuffer>()
        .init_resource::<CompIndicesBindgroup>()
        .add_systems(
            Render,
            (
                prepare_component_indices.in_set(CuttleRenderSet::PrepareIndices),
                build_component_indices_bind_group.in_set(CuttleRenderSet::PrepareBindGroups),
            )
                .chain(),
        );
}

#[derive(Debug, Clone, Copy)]
pub struct Extension {
    pub target: Entity,
    pub index: u8,
}

impl Extension {
    pub fn new(target: Entity) -> Self {
        Self { target, index: 0 }
    }
}

impl Component for Extension {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            let target = world.get::<Extension>(entity).unwrap().target;
            let mut target = world.entity_mut(target);
            match target.get_mut::<Extensions>() {
                Some(mut extensions) => {
                    let index = extensions.len();
                    extensions.push(entity);
                    world.get_mut::<Extension>(entity).unwrap().index = index as u8;
                }
                None => panic!("HI"),
            }
        });
    }
}

#[derive(Debug, Component, Clone, Deref, DerefMut, Default)]
pub struct Extensions(pub Vec<Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CompIndicesBuffer(StorageBuffer<Vec<u32>>);

#[derive(Resource, Default)]
pub struct CompIndicesBindgroup(pub Option<BindGroup>);

fn prepare_component_indices(
    mut roots: Query<(&ExtractedCuttleFlags, &Extensions, &mut RenderIndexRange)>,
    extension_flags: Query<&ExtractedCuttleFlags>,
    mut indices_buffer: ResMut<CompIndicesBuffer>,
) {
    let indices = indices_buffer.get_mut();
    indices.clear();

    for (flags, extensions, mut range) in &mut roots {
        range.end = indices.len() as u32;
        range.start = indices.len() as u32;
        range.end += flags.len() as u32;
        indices.extend(flags.iter());

        for extension_entity in extensions.iter() {
            let flags = extension_flags.get(*extension_entity).unwrap();
            range.end += flags.len() as u32;
            indices.extend(flags.iter());
        }
    }
}

fn build_component_indices_bind_group(
    mut indices_buffer: ResMut<CompIndicesBuffer>,
    mut op_bindgroup: ResMut<CompIndicesBindgroup>,
    pipeline: Res<CuttlePipeline>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    indices_buffer.write_buffer(&device, &queue);

    let entries = BindGroupEntries::sequential((indices_buffer.binding().unwrap(),));

    let bindgroup = device.create_bind_group("cuttle indices", &pipeline.op_layout, &entries);
    op_bindgroup.0 = Some(bindgroup);
}
