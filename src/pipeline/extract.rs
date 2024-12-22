use crate::components::initialization::CuttleRenderDataFrom;
use crate::groups::CuttleGroup;
use crate::prelude::Extension;
use crate::{
    bounding::CuttleBounding,
    components::{arena::IndexArena, buffer::CompBuffer},
    extensions::Extensions,
    CuttleFlags,
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
        SyncComponentPlugin::<CuttleFlags>::default(),
        ExtractComponentPlugin::<CuttleBounding>::default(),
        ExtractComponentPlugin::<ExtractedZ>::default(),
        ExtractComponentPlugin::<ExtractedVisibility>::default(),
    ))
    .sub_app_mut(RenderApp)
    .add_systems(ExtractSchedule, (extract_extensions, extract_flags));
}

pub(crate) const fn build_extract_cuttle_comp<
    G: CuttleGroup,
    C: Component,
    R: CuttleRenderDataFrom<C>,
>(
    pos: u8,
) -> impl FnMut(
    Single<&mut CompBuffer<R>>,
    Extract<Res<IndexArena<C>>>,
    Extract<Query<(&CuttleFlags, &C), (Or<(With<G>, With<Extension<G>>)>, Changed<C>)>>,
) {
    move |mut buffer, arena, comps| {
        let buffer = buffer.get_mut();
        buffer.resize_with(arena.max as usize, || R::default());

        for (flags, comp) in &comps {
            let Some(&index) = flags.indices.get(&pos) else {
                error!(
                    "Index for '{}' not set despite the component being present",
                    type_name::<C>()
                );
                continue;
            };
            let Some(elem) = buffer.get_mut(index as usize) else {
                error!(
                    "Index {} out of bounds for CompBuffer<{}> with size {}",
                    index,
                    type_name::<C>(),
                    buffer.len()
                );
                continue;
            };
            *elem = R::from_comp(comp);
        }
    }
}

fn extract_extensions(
    mut cmds: Commands,
    render_entities: Extract<Query<&RenderEntity>>,
    extend: Extract<Query<(&RenderEntity, &Extensions), Changed<Extensions>>>,
) {
    for (render, extend) in &extend {
        let render_extensions = extend
            .iter()
            .filter_map(|e| {
                render_entities
                    .get(*e)
                    .map_or_else(
                        |e| {
                            error!("Extension could not be mapped");
                            Err(e)
                        },
                        |e| Ok(e.id()),
                    )
                    .ok()
            })
            .collect();
        cmds.entity(render.id())
            .insert(Extensions(render_extensions));
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
        .map(|e| (e, (RenderIndexRange::default(), CombinedBounding::default(), G::default())))
        .collect();
    cmds.insert_or_spawn_batch(extracted)
}


#[derive(Component)]
pub(crate) struct ExtractedZ(pub f32);

impl ExtractComponent for ExtractedZ {
    type QueryData = &'static GlobalTransform;
    type QueryFilter = ();
    type Out = ExtractedZ;

    fn extract_component(
        transform: &GlobalTransform,
    ) -> Option<Self::Out> {
        Some(ExtractedZ(transform.translation().z))
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
        Some(ExtractedBounding( BoundingCircle::new(transform.translation().xy(), bounding.bounding)))
    }
}

#[derive(Component, Deref, DerefMut, Debug)]
pub(crate) struct ExtractedCuttleFlags(Vec<u32>);

fn extract_flags(mut cmds: Commands, query: Extract<Query<(RenderEntity, &CuttleFlags)>>) {
    let extracted: Vec<_> = query.iter().map(|(ent, flags)| {
        let compressed: Vec<u32> = flags.indices.iter().map(pos_and_index_to_u32).collect();
        (ent, ExtractedCuttleFlags(compressed))
    }).collect();
    cmds.insert_or_spawn_batch(extracted);
}

fn pos_and_index_to_u32((&pos, &index): (&u8, &u32)) -> u32 {
    (index << 8) | pos as u32
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
    use crate::pipeline::extract::pos_and_index_to_u32;

    #[test]
    fn test_pos_and_index_to_u32() {
        assert_eq!(0b100000001, pos_and_index_to_u32((&1,&1)));
        assert_eq!(0b10100000101, pos_and_index_to_u32((&5,&5)));
        assert_eq!(0b11111111, pos_and_index_to_u32((&255,&0)));

        let test = 0b10100000101;
        assert_eq!(5, test & 255); // Retrieve pos
        assert_eq!(5, test >> 8); // Retrieve index
    }
}