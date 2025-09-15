use crate::bounding::BoundingRadius;
use crate::bounding::GlobalBoundingCircle;
use crate::components::CuttleComponent;
use crate::components::arena::IndexArena;
use crate::components::{ExtensionIndexOverride, Positions};
use crate::configs::{ConfigStore, CuttleConfig};
use crate::extensions::{ExtendedBy, ExtensionIndex};
use crate::internal_prelude::*;
use crate::pipeline::extract::CuttleZ;
use crate::prelude::ComputeBounding;
use crate::prelude::Extends;
use bevy_camera::visibility::Visibility;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryEntityError;
use bevy_ecs::world::DeferredWorld;
use bevy_render::sync_world::SyncToRenderWorld;
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, set_flag_indices.before(ComputeBounding))
        .add_message::<AddCuttleComponent>()
        .register_type::<CuttleIndices>();
}

#[derive(Component, Reflect, Debug, Default, Deref)]
#[require(
    Visibility,
    ExtendedBy,
    BoundingRadius,
    GlobalBoundingCircle,
    CuttleZ,
    SyncToRenderWorld
)]
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
    ctx: HookContext,
) {
    let id = world.resource::<ConfigStore<G>>().id;
    world.get_mut::<CuttleIndices>(ctx.entity).unwrap().group_id = id;
}

#[derive(Debug, Component, Reflect, Deref, DerefMut, Copy, Clone)]
#[reflect(Component)]
#[component(on_add = on_add_cuttle_component_initialize_index::<C>, on_remove = on_remove_cuttle_component_release_index::<C>)]
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

fn on_add_cuttle_component_initialize_index<C: Component>(
    mut world: DeferredWorld,
    ctx: HookContext,
) {
    let index = world.resource_mut::<IndexArena<C>>().get();
    **world
        .get_mut::<CuttleComponentIndex<C>>(ctx.entity)
        .unwrap() = index;
}

fn on_remove_cuttle_component_release_index<C: Component>(
    mut world: DeferredWorld,
    ctx: HookContext,
) {
    let index = **world.get::<CuttleComponentIndex<C>>(ctx.entity).unwrap();
    world.resource_mut::<IndexArena<C>>().release(index);
}

#[derive(Reflect, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CuttleIndex {
    pub(crate) extension_index: u8,
    pub(crate) component_id: u8,
}

pub fn set_flag_indices(
    mut messages: MessageReader<AddCuttleComponent>,
    component_meta: Query<(&Positions, Option<&ExtensionIndexOverride>)>,
    extensions: Query<(&Extends, &ExtensionIndex)>,
    mut indices: Query<&mut CuttleIndices>,
) -> Result<()> {
    for message in messages.read() {
        let (positions, extension_index_override) = component_meta
            .get(message.component)
            .inspect_err(|err| println!("INDICES: {err}"))?;
        let (entity, extension_index) = match extensions.get(message.added_to) {
            Ok((&Extends(target), &ExtensionIndex(index))) => (target, index),
            Err(QueryEntityError::QueryDoesNotMatch(ent, _)) => (ent, 0),
            _ => panic!("NO ENTITY"),
        };
        let Ok(mut flags) = indices.get_mut(entity) else {
            continue;
        };
        let position = positions
            .get(flags.group_id)
            .copied()
            .ok_or("NO")?
            .ok_or("NOO")?;
        let index = CuttleIndex {
            component_id: position,
            extension_index: extension_index_override
                .map(|o| **o)
                .unwrap_or(extension_index),
        };
        flags.indices.insert(index, message.index);
    }
    Ok(())
}

#[derive(Debug, Message, Reflect)]
pub struct AddCuttleComponent {
    component: Entity,
    added_to: Entity,
    index: u32,
}

pub(crate) fn added_cuttle_component<C: Component>(
    add: On<Add, C>,
    indices: Query<&CuttleComponentIndex<C>>,
    component_meta: Single<Entity, With<CuttleComponent<C>>>,
    mut events: MessageWriter<AddCuttleComponent>,
) {
    let index = indices.get(add.entity).map(|i| i.index).unwrap_or(u32::MAX);
    events.write(AddCuttleComponent {
        component: component_meta.into_inner(),
        added_to: add.entity,
        index,
    });
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
