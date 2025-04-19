use crate::bounding::BoundingRadius;
use crate::indices::set_flag_indices;
use crate::pipeline::{CuttleRenderSet, specialization::CuttlePipeline};
use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::{
    Render, RenderApp,
    render_resource::{BindGroup, BindGroupEntries, StorageBuffer},
    renderer::{RenderDevice, RenderQueue},
};
use std::fmt::Debug;

pub fn plugin(app: &mut App) {
    app.register_type::<Extension>()
        .register_type::<Extensions>()
        .add_systems(PostUpdate, register_extensions.before(set_flag_indices));

    app.world_mut()
        .register_required_components::<Extension, BoundingRadius>();
    app.sub_app_mut(RenderApp)
        .init_resource::<CompIndicesBuffer>()
        .init_resource::<CompIndicesBindGroup>()
        .add_systems(
            Render,
            (build_component_indices_bind_group.in_set(CuttleRenderSet::PrepareBindGroups),)
                .chain(),
        );
}

#[derive(Debug, Clone, Copy, Reflect, Component)]
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

fn register_extensions(
    mut roots: Query<&mut Extensions>,
    mut leafs: Query<(Entity, &mut Extension), Added<Extension>>,
) -> Result<()> {
    for (entity, mut extension) in &mut leafs {
        let mut extensions = roots.get_mut(extension.target)?;
        extensions.push(entity);
        let index = extensions.len();
        extension.index = index as u8;
    }
    Ok(())
}

#[derive(Debug, Component, Clone, Deref, DerefMut, Default, Reflect)]
#[reflect(Component)]
pub struct Extensions(pub Vec<Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CompIndicesBuffer(StorageBuffer<Vec<u32>>);

#[derive(Resource, Default)]
pub struct CompIndicesBindGroup(pub Option<BindGroup>);

fn build_component_indices_bind_group(
    mut indices_buffer: ResMut<CompIndicesBuffer>,
    mut op_bind_group: ResMut<CompIndicesBindGroup>,
    pipeline: Res<CuttlePipeline>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    indices_buffer.write_buffer(&device, &queue);

    let entries = BindGroupEntries::sequential((indices_buffer.binding().unwrap(),));

    let bind_group = device.create_bind_group("cuttle indices", &pipeline.op_layout, &entries);
    op_bind_group.0 = Some(bind_group);
}
