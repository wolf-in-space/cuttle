use crate::bounding::BoundingRadius;
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
    app.register_type::<Extension>()
        .register_type::<Extensions>();

    app.world_mut()
        .register_required_components::<Extension, BoundingRadius>();
    app.sub_app_mut(RenderApp)
        .init_resource::<CompIndicesBuffer>()
        .init_resource::<CompIndicesBindgroup>()
        .add_systems(
            Render,
            (build_component_indices_bind_group.in_set(CuttleRenderSet::PrepareBindGroups),)
                .chain(),
        );
}

#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
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
                    extensions.push(entity);
                    let index = extensions.len();
                    world.get_mut::<Extension>(entity).unwrap().index = index as u8;
                }
                None => panic!("HI"),
            }
        });
    }
}

#[derive(Debug, Component, Clone, Deref, DerefMut, Default, Reflect)]
#[reflect(Component)]
pub struct Extensions(pub Vec<Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CompIndicesBuffer(StorageBuffer<Vec<u32>>);

#[derive(Resource, Default)]
pub struct CompIndicesBindgroup(pub Option<BindGroup>);

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
