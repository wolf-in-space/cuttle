use crate::components::initialization::CuttleRenderDataFrom;
use crate::groups::CuttleGroup;
use crate::prelude::Extension;
use crate::{
    bounding::CuttleBoundingRadius,
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

pub fn plugin(app: &mut App) {
    app.add_plugins((
        SyncComponentPlugin::<Extensions>::default(),
        ExtractComponentPlugin::<CuttleBoundingRadius>::default(),
        ExtractComponentPlugin::<CuttleFlags>::default(),
        ExtractComponentPlugin::<ExtractedVisibility>::default(),
    ))
    .sub_app_mut(RenderApp)
    .add_systems(ExtractSchedule, extract_extensions);
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

#[derive(Component)]
pub struct ExtractedCuttleTransform {
    pub bounding: BoundingCircle,
    pub z: f32,
}

#[derive(Component, Debug)]
pub struct ExtractedRenderSdf {
    pub op_start_index: u32,
    pub op_count: u32,
    pub final_bounds: BoundingCircle,
}

impl Default for ExtractedRenderSdf {
    fn default() -> Self {
        Self {
            op_count: 0,
            op_start_index: 0,
            final_bounds: BoundingCircle::new(Vec2::ZERO, 0.),
        }
    }
}

pub(crate) fn extract_group_marker<G: CuttleGroup>(
    mut cmds: Commands,
    query: Extract<Query<RenderEntity, With<G>>>,
) {
    let extracted: Vec<_> = query
        .iter()
        .map(|e| (e, (ExtractedRenderSdf::default(), G::default())))
        .collect();
    cmds.insert_or_spawn_batch(extracted)
}

impl ExtractComponent for CuttleBoundingRadius {
    type QueryData = (&'static CuttleBoundingRadius, &'static GlobalTransform);
    type QueryFilter = Or<(Changed<CuttleBoundingRadius>, Changed<GlobalTransform>)>;
    type Out = ExtractedCuttleTransform;

    fn extract_component(
        (radius, transform): (&CuttleBoundingRadius, &GlobalTransform),
    ) -> Option<Self::Out> {
        let translation = transform.translation();
        Some(ExtractedCuttleTransform {
            bounding: BoundingCircle::new(translation.xy(), radius.bounding),
            z: translation.z,
        })
    }
}

impl ExtractComponent for CuttleFlags {
    type QueryData = &'static CuttleFlags;
    type QueryFilter = Changed<CuttleFlags>;
    type Out = CuttleFlags;

    fn extract_component(internals: &Self) -> Option<Self::Out> {
        Some(internals.clone())
    }
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
