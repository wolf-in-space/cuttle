use crate::pipeline::extract::ExtractedSdfTransform;
use crate::pipeline::{extract::ExtractedRenderSdf, specialization::SdfPipeline, CuttleRenderSet};
use crate::SdfInternals;
use bevy::math::bounding::BoundingVolume;
use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroup, BindGroupEntries, ShaderType, StorageBuffer},
        renderer::{RenderDevice, RenderQueue},
        sync_world::SyncToRenderWorld,
        Render, RenderApp,
    },
};
use std::fmt::{self, Debug};

pub fn plugin(app: &mut App) {
    register_extend_sdf_hooks(app.world_mut());

    app.sub_app_mut(RenderApp)
        .init_resource::<OpsBuffer>()
        .init_resource::<CompIndicesBuffer>()
        .init_resource::<OpBindgroup>()
        .add_systems(
            Render,
            (
                build_op_buffer.in_set(CuttleRenderSet::OpPreparation),
                build_op_bindgroups.in_set(CuttleRenderSet::PrepareBindgroups),
            )
                .chain(),
        );
}

#[derive(Debug, Component, Clone, Copy)]
#[require(SdfInternals, SyncToRenderWorld)]
pub struct ExtendSdf {
    target: Entity,
}

impl ExtendSdf {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

fn register_extend_sdf_hooks(world: &mut World) {
    world
        .register_component_hooks::<ExtendSdf>()
        .on_add(|mut world, entity, _| {
            let target = world.get::<ExtendSdf>(entity).unwrap().target;
            let mut target = world.entity_mut(target);
            match target.get_mut::<SdfExtensions>() {
                Some(mut extensions) => extensions.push(entity),
                None => panic!("HI"),
            }
        });
}

#[derive(Debug, Component, Clone, Copy)]
pub enum BoundingEffect {
    Nothing,
    Combine,
}

#[derive(Debug, Component, Clone, Deref, DerefMut, Default)]
pub struct SdfExtensions(pub Vec<Entity>);

#[derive(ShaderType, Clone, Copy)]
pub struct Op {
    pub start_index: u32,
    pub flag: u32,
}

impl Debug for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Op[start={},flag={:b}]",
            self.start_index, self.flag
        ))
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct OpsBuffer(StorageBuffer<Vec<Op>>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CompIndicesBuffer(StorageBuffer<Vec<u32>>);

#[derive(Resource, Default)]
pub struct OpBindgroup(pub Option<BindGroup>);

fn build_op_buffer(
    mut sdfs: Query<(
        &SdfInternals,
        &ExtractedSdfTransform,
        &mut ExtractedRenderSdf,
        &SdfExtensions,
    )>,
    extracted: Query<(&SdfInternals, &ExtractedSdfTransform)>,
    mut ops_buffer: ResMut<OpsBuffer>,
    mut indices_buffer: ResMut<CompIndicesBuffer>,
) {
    let indices = indices_buffer.get_mut();
    indices.clear();
    let ops = ops_buffer.get_mut();
    ops.clear();

    let mut add_op = |ops: &mut Vec<Op>, sdf: &SdfInternals| {
        let op = Op {
            start_index: indices.len() as u32,
            flag: sdf.flag.0,
        };
        indices.extend(sdf.indices.values());
        ops.push(op);
    };

    for (sdf, transform, mut render_sdf, extensions) in &mut sdfs {
        render_sdf.op_count = extensions.len() as u32 + 1;
        render_sdf.op_start_index = ops.len() as u32;
        render_sdf.final_bounds = transform.bounding;

        add_op(ops, sdf);

        for extension_entity in extensions.iter() {
            let (sdf, transform) = extracted.get(*extension_entity).unwrap();
            render_sdf.final_bounds = render_sdf.final_bounds.merge(&transform.bounding);
            add_op(ops, sdf);
        }
    }
}

fn build_op_bindgroups(
    mut ops_buffer: ResMut<OpsBuffer>,
    mut indices_buffer: ResMut<CompIndicesBuffer>,
    mut op_bindgroup: ResMut<OpBindgroup>,
    pipeline: Res<SdfPipeline>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    ops_buffer.write_buffer(&device, &queue);
    indices_buffer.write_buffer(&device, &queue);

    let entries = BindGroupEntries::sequential((
        ops_buffer.binding().unwrap(),
        indices_buffer.binding().unwrap(),
    ));

    let bindgroup = device.create_bind_group("sdf operations", &pipeline.op_layout, &entries);
    op_bindgroup.0 = Some(bindgroup);
}
