use crate::bounding::BoundingRadius;
use crate::indices::set_flag_indices;
use crate::pipeline::{specialization::CuttlePipeline, CuttleRenderSet};
use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::{
    render_resource::{BindGroup, BindGroupEntries, StorageBuffer}, renderer::{RenderDevice, RenderQueue},
    Render,
    RenderApp,
};
use std::fmt::Debug;

pub fn plugin(app: &mut App) {
    app.register_type::<Extends>()
        .register_type::<ExtendedBy>()
        .add_systems(PostUpdate, set_extension_index.before(set_flag_indices));

    app.world_mut()
        .register_required_components::<Extends, BoundingRadius>();
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
#[relationship(relationship_target = ExtendedBy)]
#[require(ExtensionIndex)]
#[reflect(Component)]
pub struct Extends(pub Entity);

#[derive(Debug, Default, Clone, Copy, Reflect, Component)]
#[reflect(Component)]
pub struct ExtensionIndex(pub(crate) u8);

fn set_extension_index(
    roots: Query<&ExtendedBy>,
    mut leafs: Query<(&Extends, &mut ExtensionIndex), Added<Extends>>,
) -> Result<()> {
    for (extension, mut target) in &mut leafs {
        target.0 = roots.get(extension.0)?.len() as u8;
    }
    Ok(())
}

#[derive(Debug, Component, Clone, Deref, DerefMut, Default, Reflect)]
#[relationship_target(relationship = Extends)]
#[reflect(Component)]
pub struct ExtendedBy(Vec<Entity>);

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
