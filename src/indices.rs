use crate::bounding::BoundingRadius;
use crate::bounding::GlobalBoundingCircle;
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

#[derive(Component, Reflect, Debug, Default, Deref)]
#[require(Visibility, Extensions, BoundingRadius, GlobalBoundingCircle)]
#[reflect(Component)]
pub struct CuttleIndices {
    #[deref]
    pub(crate) indices: BTreeMap<CuttleIndex, u32>,
    pub(crate) group_id: usize,
}

impl CuttleIndices {
    pub fn iter_as_packed_u32s(&self) -> impl Iterator<Item = u32> + '_ {
        self.indices.iter().map(Self::id_and_index_to_u32)
    }

    fn id_and_index_to_u32(
        (&CuttleIndex { component_id, .. }, &index): (&CuttleIndex, &u32),
    ) -> u32 {
        (index << 8) | component_id as u32
    }

    pub fn group_id(&self) -> usize {
        self.group_id
    }
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
pub(crate) struct CuttleComponentIndex<C: Component> {
    #[deref]
    index: u32,
    #[reflect(ignore)]
    phantom: PhantomData<C>,
}

impl<C: Component> Default for CuttleComponentIndex<C> {
    fn default() -> Self {
        Self {
            index: 0,
            phantom: PhantomData,
        }
    }
}

impl<C: Component> Component for CuttleComponentIndex<C> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        let on_add = |mut world: DeferredWorld, entity, _| {
            let index = world.resource_mut::<IndexArena<C>>().get();
            **world.get_mut::<CuttleComponentIndex<C>>(entity).unwrap() = index;
        };
        let on_remove = |mut world: DeferredWorld, entity, _| {
            let index = **world.get::<CuttleComponentIndex<C>>(entity).unwrap();
            world.resource_mut::<IndexArena<C>>().release(index);
        };
        hooks.on_add(on_add).on_remove(on_remove);
    }
}

#[derive(Reflect, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CuttleIndex {
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
    let index = world
        .get::<CuttleComponentIndex<C>>(entity)
        .map(|i| **i)
        .unwrap_or(u32::MAX);

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

#[cfg(test)]
mod tests {
    use crate::indices::{CuttleIndex, CuttleIndices};

    #[test]
    fn test_pos_and_index_to_u32() {
        assert_eq!(
            0b100000001,
            CuttleIndices::id_and_index_to_u32((
                &CuttleIndex {
                    component_id: 1,
                    extension_index: 0
                },
                &1
            ))
        );
        assert_eq!(
            0b10100000101,
            CuttleIndices::id_and_index_to_u32((
                &CuttleIndex {
                    component_id: 5,
                    extension_index: 0
                },
                &5
            ))
        );
        assert_eq!(
            0b11111111,
            CuttleIndices::id_and_index_to_u32((
                &CuttleIndex {
                    component_id: 255,
                    extension_index: 0
                },
                &0
            ))
        );

        let test = 0b10100000101;
        assert_eq!(5, test & 255); // Retrieve pos
        assert_eq!(5, test >> 8); // Retrieve index
    }
}
