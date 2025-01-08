use crate::components::initialization::CuttleRenderDataFrom;
use crate::groups::CuttleGroup;
use crate::indices::{ComponentIndex, CuttleIndex, CuttleIndices};
use crate::{
    bounding::CuttleBounding,
    components::{arena::IndexArena, buffer::CompBuffer},
    extensions::Extensions,
};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::{
    math::bounding::BoundingCircle,
    prelude::*,
    render::{sync_component::SyncComponentPlugin, sync_world::RenderEntity, Extract, RenderApp},
};
use std::any::type_name;
use std::fmt::Debug;
use std::ops::Range;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        SyncComponentPlugin::<Extensions>::default(),
        SyncComponentPlugin::<CuttleIndices>::default(),
        ExtractComponentPlugin::<CuttleBounding>::default(),
        ExtractComponentPlugin::<ExtractedZ>::default(),
        ExtractComponentPlugin::<ExtractedVisibility>::default(),
    ))
    .sub_app_mut(RenderApp)
    .add_systems(ExtractSchedule, (extract_flags));
}

pub(crate) fn extract_cuttle_comp<C: Component, R: CuttleRenderDataFrom<C>>(
    mut buffer: Single<&mut CompBuffer<R>>,
    arena: Extract<Res<IndexArena<C>>>,
    comps: Extract<Query<(&ComponentIndex<C>, &C), Changed<C>>>,
) {
    let buffer = buffer.get_mut();
    buffer.resize_with(arena.max as usize, || R::default());

    for (index, comp) in &comps {
        if let Some(elem) = buffer.get_mut(**index as usize) {
            *elem = R::from_comp(comp);
        } else {
            error!(
                "{} out of bounds for CompBuffer<{}> with size {}",
                **index,
                type_name::<C>(),
                buffer.len()
            );
        };
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub(crate) struct RenderIndexRange(pub Range<u32>);

#[derive(Component, Debug, Deref, DerefMut)]
pub(crate) struct CombinedBounding(pub BoundingCircle);

impl Default for CombinedBounding {
    fn default() -> Self {
        Self(BoundingCircle::new(Vec2::ZERO, 0.))
    }
}

pub(crate) fn extract_group_marker<G: CuttleGroup>(
    mut cmds: Commands,
    query: Extract<Query<RenderEntity, With<G>>>,
) {
    let extracted: Vec<_> = query
        .iter()
        .map(|e| {
            (
                e,
                (
                    RenderIndexRange::default(),
                    CombinedBounding::default(),
                    G::default(),
                ),
            )
        })
        .collect();
    cmds.insert_or_spawn_batch(extracted)
}

#[derive(Component)]
pub(crate) struct ExtractedZ(pub f32);

impl ExtractComponent for ExtractedZ {
    type QueryData = (&'static GlobalTransform, Option<&'static ComputedNode>);
    type QueryFilter = ();
    type Out = ExtractedZ;

    fn extract_component(
        (transform, z_index): (&GlobalTransform, Option<&ComputedNode>),
    ) -> Option<Self::Out> {
        Some(ExtractedZ(match z_index {
            None => transform.translation().z,
            Some(computed) => computed.stack_index() as f32,
        }))
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct ExtractedBounding(pub(crate) BoundingCircle);

impl ExtractComponent for CuttleBounding {
    type QueryData = (&'static CuttleBounding, &'static GlobalTransform);
    type QueryFilter = ();
    type Out = ExtractedBounding;

    fn extract_component(
        (bounding, transform): (&CuttleBounding, &GlobalTransform),
    ) -> Option<Self::Out> {
        Some(ExtractedBounding(BoundingCircle::new(
            transform.translation().xy(),
            bounding.bounding,
        )))
    }
}

#[derive(Component, Deref, DerefMut, Debug)]
pub(crate) struct ExtractedCuttleFlags(Vec<u32>);

fn extract_flags(mut cmds: Commands, query: Extract<Query<(RenderEntity, &CuttleIndices)>>) {
    let extracted: Vec<_> = query
        .iter()
        .map(|(ent, flags)| {
            let compressed: Vec<u32> = flags.indices.iter().map(id_and_index_to_u32).collect();
            (ent, ExtractedCuttleFlags(compressed))
        })
        .collect();
    cmds.insert_or_spawn_batch(extracted);
}

fn id_and_index_to_u32((&CuttleIndex { component_id, .. }, &index): (&CuttleIndex, &u32)) -> u32 {
    (index << 8) | component_id as u32
}

#[derive(Component)]
pub struct ExtractedVisibility(pub bool);

impl ExtractComponent for ExtractedVisibility {
    type QueryData = &'static ViewVisibility;
    type QueryFilter = ();
    type Out = ExtractedVisibility;

    fn extract_component(vis: &ViewVisibility) -> Option<Self::Out> {
        Some(ExtractedVisibility(vis.get()))
    }
}

#[cfg(test)]
mod tests {
    use crate::indices::CuttleIndex;
    use crate::pipeline::extract::id_and_index_to_u32;

    #[test]
    fn test_pos_and_index_to_u32() {
        assert_eq!(
            0b100000001,
            id_and_index_to_u32((
                &CuttleIndex {
                    component_id: 1,
                    extension_index: 0
                },
                &1
            ))
        );
        assert_eq!(
            0b10100000101,
            id_and_index_to_u32((
                &CuttleIndex {
                    component_id: 5,
                    extension_index: 0
                },
                &5
            ))
        );
        assert_eq!(
            0b11111111,
            id_and_index_to_u32((
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
