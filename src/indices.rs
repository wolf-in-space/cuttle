use crate::bounding::CuttleBounding;
use crate::components::arena::IndexArena;
use crate::extensions::Extensions;
use crate::groups::{CuttleGroup, GroupIdStore};
use crate::prelude::Extension;
use bevy::ecs::component::{ComponentHooks, ComponentId, StorageType};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CuttleIndices>();
}

#[derive(Component, Reflect, Debug, Default)]
#[require(Transform, Visibility, Extensions, CuttleBounding)]
#[reflect(Component)]
pub struct CuttleIndices {
    pub(crate) indices: BTreeMap<CuttleIndex, u32>,
    group_id: usize,
}

pub fn on_add_group_marker_initialize_indices_group_id<G: CuttleGroup>(
    mut world: DeferredWorld,
    entity: Entity,
    _: ComponentId,
) {
    let id = world.resource::<GroupIdStore<G>>().id;
    world.get_mut::<CuttleIndices>(entity).unwrap().group_id = id;
}

#[derive(Debug, Reflect, Deref, DerefMut, Copy, Clone)]
#[reflect(Component)]
pub(crate) struct ComponentIndex<C: Component> {
    #[deref]
    index: u32,
    #[reflect(ignore)]
    phantom: PhantomData<C>,
}

impl<C: Component> Default for ComponentIndex<C> {
    fn default() -> Self {
        Self {
            index: 0,
            phantom: PhantomData,
        }
    }
}

impl<C: Component> Component for ComponentIndex<C> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        let on_add = |mut world: DeferredWorld, entity, _| {
            let index = world.resource_mut::<IndexArena<C>>().get();
            **world.get_mut::<ComponentIndex<C>>(entity).unwrap() = index;
        };
        let on_remove = |mut world: DeferredWorld, entity, _| {
            let index = **world.get::<ComponentIndex<C>>(entity).unwrap();
            world.resource_mut::<IndexArena<C>>().release(index);
        };
        hooks.on_add(on_add).on_remove(on_remove);
    }
}

#[derive(Reflect, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CuttleIndex {
    pub(crate) extension_index: u8,
    pub(crate) component_id: u8,
}

pub(crate) const fn build_set_flag_index<const SET: bool, T, C: Component>(
    positions: Vec<Option<u8>>,
) -> impl FnMut(Trigger<T, C>, DeferredWorld) {
    move |trigger, world| {
        if SET {
            set_index::<C>(&positions, world, trigger.entity())
        } else {
            remove_index::<C>(&positions, world, trigger.entity())
        }
    }
}

fn set_index<C: Component>(positions: &[Option<u8>], mut world: DeferredWorld, entity: Entity) {
    let index = **world.get::<ComponentIndex<C>>(entity).unwrap();

    let Some((flags, pos)) = get_indices_and_pos(&mut world, positions, entity) else {
        return;
    };

    flags.indices.insert(pos, index);
}

fn remove_index<C: Component>(positions: &[Option<u8>], mut world: DeferredWorld, entity: Entity) {
    let Some((flags, index)) = get_indices_and_pos(&mut world, positions, entity) else {
        return;
    };

    if let None = flags.indices.remove(&index) {
        error!("Tried to remove an index that no longer exists")
    }
}

fn get_indices_and_pos<'a>(
    world: &'a mut DeferredWorld,
    positions: &[Option<u8>],
    entity: Entity,
) -> Option<(&'a mut CuttleIndices, CuttleIndex)> {
    let (entity, extension_index) = match world.get::<Extension>(entity) {
        Some(&Extension { target, index, .. }) => (target, index),
        None => (entity, 0),
    };
    let flags = world.get_mut::<CuttleIndices>(entity)?;
    let pos = positions.get(flags.group_id).copied()??;

    Some((
        flags.into_inner(),
        CuttleIndex {
            component_id: pos,
            extension_index,
        },
    ))
}
