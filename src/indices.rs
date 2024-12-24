use bevy::prelude::*;
use std::collections::BTreeMap;
use bevy::ecs::component::ComponentId;
use bevy::ecs::world::DeferredWorld;
use crate::extensions::Extensions;
use crate::bounding::CuttleBounding;
use crate::components::arena::IndexArena;
use crate::groups::{CuttleGroup, GroupId};

#[derive(Component, Debug)]
#[component(on_add=on_add_indices)]
#[require(Transform, Visibility, Extensions, CuttleBounding)]
pub struct CuttleIndices {
    pub(crate) indices: BTreeMap<u8, u32>,
    pub(crate) retrieve_group_id: fn(&DeferredWorld) -> usize,
    pub(crate) group_id: usize,
}

fn on_add_indices(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let indices = world.get::<CuttleIndices>(entity).unwrap();
    let id = (indices.retrieve_group_id)(&world);
    world.get_mut::<CuttleIndices>(entity).unwrap().group_id = id;
}

impl CuttleIndices {
    pub(crate) const fn new<G: CuttleGroup>() -> Self {
        Self {
            retrieve_group_id: |world| {
                world.resource::<GroupId<G>>().id
            },
            indices: BTreeMap::new(),
            group_id: usize::MAX,
        }
    }
}

pub(crate) const fn build_set_flag_index<const SET: bool, T, C: Component>(positions: Vec<Option<u8>>)
   -> impl FnMut(Trigger<T, C>, DeferredWorld) {
    move |trigger, world| {
        if SET {
            set_index::<C>(&positions, world, trigger.entity())
        } else {
            remove_index::<C>(&positions, world, trigger.entity())
        }
    }
}

fn set_index<C: Component>(positions: &[Option<u8>], mut world: DeferredWorld, entity: Entity) {
    let index = world.resource_mut::<IndexArena<C>>().get();

    let Some((flags, pos)) = get_indices_and_pos(&mut world, positions, entity) else {
        return;
    };

    flags.indices.insert(pos, index);
}

fn remove_index<C: Component>(positions: &[Option<u8>], mut world: DeferredWorld, entity: Entity) {
    let Some((flags, pos)) = get_indices_and_pos(&mut world, positions, entity) else {
        return;
    };
    
    if let Some(index) = flags.indices.remove(&pos) {
        world.resource_mut::<IndexArena<C>>().release(index);
    } else {
        error!("Tried to remove an index that no longer exists")
    }
}

fn get_indices_and_pos<'a>(world: &'a mut DeferredWorld, positions: &[Option<u8>], entity: Entity) -> Option<(&'a mut CuttleIndices, u8)> {
    let Some(flags) = world.get_mut::<CuttleIndices>(entity) else {
        return None;
    };
    let Some(pos) = positions.get(flags.group_id).copied().flatten() else {
        return None;
    };
    
    Some((flags.into_inner(), pos))
}