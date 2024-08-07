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
    pub operations: Operations,
    pub flags: SdfFlags,
    pub indices: SdfBufferIndices,
    pub sdf: SdfBundle,
    pub combined: CombinedAABB,
}

impl RenderSdfBundle {
    pub fn new() -> Self {
        default()
    }

    pub fn with_pos(self, pos: impl Into<Vec2>) -> Self {
        Self {
            sdf: self.sdf.with_pos(pos),
            ..self
        }
    }

    pub fn with_z_index(self, index: f32) -> Self {
        Self {
            sdf: self.sdf.with_z_index(index),
            ..self
        }
    }

    pub fn with_rot(self, rot: f32) -> Self {
        Self {
            sdf: self.sdf.with_rot(rot),
            ..self
        }
    }
}

#[derive(Bundle, Default)]
pub struct SdfBundle {
    pub transform: TransformBundle,
    pub binding: SdfBinding,
    pub index: SdfBufferIndex,
    pub flag: Flag<Comp>,
    pub aabb: AABB,
    pub size: SdfSize,
}

impl SdfBundle {
    pub fn with_pos(self, pos: impl Into<Vec2>) -> Self {
        let pos = pos.into();

        Self {
            transform: TransformBundle {
                local: Transform {
                    translation: Vec3 {
                        x: pos.x,
                        y: pos.y,
                        ..self.transform.local.translation
                    },
                    ..self.transform.local
                },
                ..self.transform
            },
            ..self
        }
    }

    pub fn with_z_index(self, index: f32) -> Self {
        Self {
            transform: TransformBundle {
                local: Transform {
                    translation: Vec3 {
                        z: index,
                        ..self.transform.local.translation
                    },
                    ..self.transform.local
                },
                ..self.transform
            },
            ..self
        }
    }

    pub fn with_rot(self, rot: f32) -> Self {
        Self {
            transform: TransformBundle {
                local: self
                    .transform
                    .local
                    .with_rotation(Quat::from_rotation_z(rot)),
                ..self.transform
            },
            ..self
        }
    }
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
        let origin = self.spawn(bundle).id();

        SdfOperationSpawner {
            cmds: self,
            origin,
            order: 10,
        }
    }
}
