use crate::bounding::GlobalBoundingCircle;
use crate::components::initialization::CuttleRenderDataFrom;
use crate::components::{arena::IndexArena, buffer::CompBuffer};
use crate::extensions::CompIndicesBuffer;
use crate::indices::{ComponentIndex, CuttleIndex, CuttleIndices};
use bevy::ecs::entity::EntityHashMap;
use bevy::{
    math::bounding::BoundingCircle,
    prelude::*,
    render::{Extract, RenderApp},
};
use std::any::type_name;
use std::fmt::Debug;

pub fn plugin(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<ExtractedCuttles>()
        .add_systems(ExtractSchedule, extract_cuttles);
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

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct ExtractedCuttles(EntityHashMap<ExtractedCuttle>);

#[derive(Debug)]
pub struct ExtractedCuttle {
    pub group_id: usize,
    pub visible: bool,
    pub bounding: BoundingCircle,
    pub indices_start: u32,
    pub indices_end: u32,
    pub z: f32,
}

fn extract_cuttles(
    extract: Extract<
        Query<(
            Entity,
            &GlobalTransform,
            &GlobalBoundingCircle,
            &CuttleIndices,
            &ViewVisibility,
        )>,
    >,
    mut extracted: ResMut<ExtractedCuttles>,
    mut buffer: ResMut<CompIndicesBuffer>,
) {
    let buffer = buffer.get_mut();
    buffer.clear();

    **extracted = extract
        .iter()
        .map(|(entity, transform, bounding, indices, vis)| {
            let indices_start = buffer.len() as u32;
            let indices_end = (buffer.len() + indices.indices.len()) as u32;
            buffer.extend(indices.iter_as_packed_u32s());

            (
                entity,
                ExtractedCuttle {
                    group_id: indices.group_id,
                    visible: **vis,
                    indices_start,
                    indices_end,
                    bounding: **bounding,
                    z: transform.translation().z,
                },
            )
        })
        .inspect(|e| {
            dbg!(e);
        })
        .collect();
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
