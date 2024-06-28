use crate::{
    flag::{BitPosition, Flag, FlagStorage, Op},
    shader::lines::Lines,
    utils::GetOrInitResourceWorldExt,
};
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use std::{any::type_name, array::from_fn};

pub fn plugin(app: &mut App) {
    app.init_resource::<OperationInfos>();
}

pub trait RegisterSdfRenderOpAppExt {
    fn register_sdf_render_operation<Op: Operation>(&mut self) -> &mut Self;
}

impl RegisterSdfRenderOpAppExt for App {
    fn register_sdf_render_operation<O: Operation>(&mut self) -> &mut Self {
        let world = self.world_mut();
        let mut infos = world.resource_or_init::<OperationInfos>();
        let bit = infos.register(O::operation_info());
        world.insert_resource(BitPosition::<O>::new(bit));

        trace!(
            "Registered op {}: pos={}, {:#?}",
            type_name::<O>(),
            bit,
            O::operation_info()
        );

        self
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Operations(EntityHashMap<OperationEntry>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationEntry {
    pub order: usize,
    pub operation: Flag<Op>,
}

impl OperationEntry {
    pub fn new(order: usize, operation: Flag<Op>) -> Self {
        Self { order, operation }
    }
}

#[derive(Component)]
pub struct OperationTarget {
    pub(crate) _targeted_by: EntityHashMap<f32>,
}

impl OperationTarget {
    pub fn single(entity: Entity, value: f32) -> Self {
        Self {
            _targeted_by: EntityHashMap::from_iter([(entity, value)]),
        }
    }
}

pub type OperationInfos = FlagStorage<OperationInfo, { Flag::<Op>::SIZE }>;

impl Default for OperationInfos {
    fn default() -> Self {
        Self {
            storage: from_fn(|i| OperationInfo {
                value: None,
                snippets: Lines::default(),
                operation: format!("UNINITIALIZED{i}"),
            }),
            count: 0,
        }
    }
}

#[derive(Debug)]
pub struct OperationInfo {
    pub value: Option<(&'static str, f32)>,
    pub snippets: Lines,
    pub operation: String,
}

pub trait Operation: Send + Sync + 'static {
    fn operation_info() -> OperationInfo;
}
