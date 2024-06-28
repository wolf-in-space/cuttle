/*
THE PLAN:

cmds.sdf((Point, Added(10.), Fill()))

cmds.sdf((Point, Added(10.), Fill()))
.operation::<Union>((Line, Added(5.)))
.operation::<SmoothSubtract>((Line, Added(5.), Transform::default()))

*/

use crate::{
    aabb::CombinedAABB,
    components::extract::{SdfBinding, SdfBufferIndex, SdfBufferIndices},
    flag::{BitPosition, Comp, Flag, Op, SdfFlags},
    operations::{Operation, OperationEntry, OperationTarget, Operations},
};
use bevy::prelude::*;
use bevy_comdf_core::aabb::{SdfSize, AABB};
use std::any::type_name;

#[derive(Bundle, Default)]
pub struct RenderSdfBundle {
    operations: Operations,
    flags: SdfFlags,
    indices: SdfBufferIndices,
    sdf: SdfBundle,
    combined: CombinedAABB,
}

#[derive(Bundle, Default)]
pub struct SdfBundle {
    binding: SdfBinding,
    index: SdfBufferIndex,
    flag: Flag<Comp>,
    aabb: AABB,
    size: SdfSize,
}

pub struct SdfOperationSpawner<'a, 'b, 'c> {
    cmds: &'c mut Commands<'a, 'b>,
    origin: Entity,
    order: usize,
}

impl<'a, 'b, 'c> SdfOperationSpawner<'a, 'b, 'c> {
    pub fn operation<O: Operation>(mut self, bundle: impl Bundle) -> Self {
        let op_entity = self
            .cmds
            .spawn((
                bundle,
                SdfBundle::default(),
                OperationTarget::single(
                    self.origin,
                    O::operation_info()
                        .value
                        .map(|(_, v)| v)
                        .unwrap_or_default(),
                ),
            ))
            .id();

        self.cmds.add(move |world: &mut World| {
            match world
                .get_resource::<BitPosition<O>>()
                .map(|p| p.as_flag::<Op>())
            {
                None => error!("Resources for Sdf Operation {} not found, you probably need to register it with app.register_sdf_render_operation", type_name::<Op>()),
                Some(flag) => {
                    trace!("Insert op: {} with flag {flag:?}", type_name::<O>());
                    world
                        .get_mut::<Operations>(self.origin)
                        .unwrap()
                        .insert(op_entity, OperationEntry::new(self.order, flag));
                }
            }
        });

        self.order += 10;
        self
    }
}

pub trait SpawnSdfCmdExt<'a, 'b, 'c> {
    fn sdf(&'c mut self, bundle: impl Bundle) -> SdfOperationSpawner<'a, 'b, 'c>;
}

impl<'a, 'b, 'c> SpawnSdfCmdExt<'a, 'b, 'c> for Commands<'a, 'b> {
    fn sdf(&'c mut self, bundle: impl Bundle) -> SdfOperationSpawner<'a, 'b, 'c> {
        let origin = self.spawn((bundle, RenderSdfBundle::default())).id();

        SdfOperationSpawner {
            cmds: self,
            origin,
            order: 10,
        }
    }
}
