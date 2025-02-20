use crate::bounding::BoundingRadius;
use crate::bounding::GlobalBoundingCircle;
use crate::components::arena::IndexArena;
use crate::components::{ExtensionIndexOverride, Positions};
use crate::configs::{ConfigStore, CuttleConfig};
use crate::extensions::Extensions;
use crate::internal_prelude::*;
use crate::pipeline::extract::CuttleZ;
use crate::prelude::Extension;
use bevy_ecs::component::{ComponentHooks, ComponentId, StorageType};
use bevy_ecs::world::DeferredWorld;
use bevy_utils::tracing::error;
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CuttleIndices>();
}

#[derive(Component, Reflect, Debug, Default, Deref)]
#[require(Visibility, Extensions, BoundingRadius, GlobalBoundingCircle, CuttleZ)]
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

pub fn on_add_config_marker_initialize_indices_config_id<G: CuttleConfig>(
    mut world: DeferredWorld,
    entity: Entity,
    _: ComponentId,
) {
    let id = world.resource::<ConfigStore<G>>().id;
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

pub fn init_component_observers(
    mut cmds: Commands,
    query: Query<(
        &Positions,
        Option<&ExtensionIndexOverride>,
        &InitObserversFn,
    )>,
) {
    for (positions, index_override, init_fn) in &query {
        init_fn.0(&mut cmds, positions.clone(), index_override.map(|o| o.0))
    }
}

pub fn init_observers<C: Component>(
    cmds: &mut Commands,
    positions: Positions,
    extension_index_override: Option<u8>,
) {
    if let Some(index_override) = extension_index_override {
        cmds.add_observer(build_set_flag_index::<true, true, OnAdd, C>(
            positions.clone(),
            index_override,
        ));
        cmds.add_observer(build_set_flag_index::<false, true, OnRemove, C>(
            positions,
            index_override,
        ));
    } else {
        cmds.add_observer(build_set_flag_index::<true, false, OnAdd, C>(
            positions.clone(),
            0,
        ));
        cmds.add_observer(build_set_flag_index::<false, false, OnRemove, C>(
            positions, 0,
        ));
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct InitObserversFn(pub fn(&mut Commands, Positions, Option<u8>));

pub(crate) const fn build_set_flag_index<const SET: bool, const OVERRIDE: bool, T, C: Component>(
    positions: Positions,
    extension_index_override: u8,
) -> impl FnMut(Trigger<T, C>, DeferredWorld) {
    move |trigger, world| {
        if SET {
            set_index::<OVERRIDE, C>(
                &positions,
                extension_index_override,
                world,
                trigger.entity(),
            )
        } else {
            remove_index::<OVERRIDE>(
                &positions,
                extension_index_override,
                world,
                trigger.entity(),
            )
        }
    }
}

fn set_index<const OVERRIDE: bool, C: Component>(
    positions: &Positions,
    extension_index_override: u8,
    mut world: DeferredWorld,
    entity: Entity,
) {
    let index = world
        .get::<CuttleComponentIndex<C>>(entity)
        .map(|i| **i)
        .unwrap_or(u32::MAX);

    let Some((flags, pos)) =
        get_indices_and_pos::<OVERRIDE>(&mut world, positions, extension_index_override, entity)
    else {
        return;
    };

    flags.indices.insert(pos, index);
}

fn remove_index<const OVERRIDE: bool>(
    positions: &Positions,
    extension_index_override: u8,
    mut world: DeferredWorld,
    entity: Entity,
) {
    let Some((flags, index)) =
        get_indices_and_pos::<OVERRIDE>(&mut world, positions, extension_index_override, entity)
    else {
        return;
    };

    if flags.indices.remove(&index).is_none() {
        error!("Tried to remove an index that no longer exists")
    }
}

fn get_indices_and_pos<'a, const OVERRIDE: bool>(
    world: &'a mut DeferredWorld,
    positions: &Positions,
    extension_index_override: u8,
    entity: Entity,
) -> Option<(&'a mut CuttleIndices, CuttleIndex)> {
    let (entity, extension_index) = match world.get::<Extension>(entity) {
        Some(&Extension { target, index, .. }) => (target, index),
        None => (entity, 0),
    };
    let flags = world.get_mut::<CuttleIndices>(entity)?;
    let position = positions.get(flags.group_id).copied()??;

    Some((
        flags.into_inner(),
        CuttleIndex {
            component_id: position,
            extension_index: if OVERRIDE {
                extension_index_override
            } else {
                extension_index
            },
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
