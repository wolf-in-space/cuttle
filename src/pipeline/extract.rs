use crate::components::initialization::SdfRenderDataFrom;
use crate::groups::SdfGroup;
use crate::{
    bounding::SdfBoundingRadius,
    components::{arena::IndexArena, buffer::CompBuffer},
    extensions::SdfExtensions,
    SdfInternals,
};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::{
    math::bounding::BoundingCircle,
    prelude::*,
    render::{sync_component::SyncComponentPlugin, sync_world::RenderEntity, Extract, RenderApp},
};
use std::fmt::Debug;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        SyncComponentPlugin::<SdfExtensions>::default(),
        ExtractComponentPlugin::<SdfBoundingRadius>::default(),
        ExtractComponentPlugin::<SdfInternals>::default(),
        ExtractComponentPlugin::<ExtractedVisibility>::default(),
    ))
    .sub_app_mut(RenderApp)
    .add_systems(ExtractSchedule, extract_sdf_extensions);
}

pub(crate) const fn build_extract_sdf_comp<C: Component, R: SdfRenderDataFrom<C>>(
    pos: u8,
) -> impl FnMut(
    Single<&mut CompBuffer<R>>,
    Extract<Res<IndexArena<C>>>,
    Extract<Query<(&SdfInternals, &C), Changed<C>>>,
) {
    move |mut buffer, arena, comps| {
        let buffer = buffer.get_mut();
        buffer.resize_with(arena.max as usize, || R::default());

        for (sdf, comp) in &comps {
            let index = *sdf.indices.get(&pos).unwrap() as usize;
            let elem = buffer.get_mut(index).unwrap();
            *elem = R::from_sdf_comp(comp);
        }
    }
}

fn extract_sdf_extensions(
    mut cmds: Commands,
    render_entities: Extract<Query<&RenderEntity>>,
    extend: Extract<Query<(&RenderEntity, &SdfExtensions), Changed<SdfExtensions>>>,
) {
    for (render, extend) in &extend {
        let render_extensions = extend
            .iter()
            .filter_map(|e| {
                render_entities
                    .get(*e)
                    .map_or_else(
                        |e| {
                            warn!("SdfExtension");
                            Err(e)
                        },
                        |e| Ok(e.id()),
                    )
                    .ok()
            })
            .collect();
        cmds.entity(render.id())
            .insert(SdfExtensions(render_extensions));
    }
}

#[derive(Component)]
pub struct ExtractedSdfTransform {
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

pub(crate) fn extract_group_marker<G: SdfGroup>(
    mut cmds: Commands,
    query: Extract<Query<RenderEntity, With<G>>>,
) {
    let extracted: Vec<_> = query
        .iter()
        .map(|e| (e, ExtractedRenderSdf::default()))
        .collect();
    cmds.insert_or_spawn_batch(extracted)
}

impl ExtractComponent for SdfBoundingRadius {
    type QueryData = (&'static SdfBoundingRadius, &'static GlobalTransform);
    type QueryFilter = Or<(Changed<SdfBoundingRadius>, Changed<GlobalTransform>)>;
    type Out = ExtractedSdfTransform;

    fn extract_component(
        (radius, transform): (&SdfBoundingRadius, &GlobalTransform),
    ) -> Option<Self::Out> {
        let translation = transform.translation();
        Some(ExtractedSdfTransform {
            bounding: BoundingCircle::new(translation.xy(), radius.bounding),
            z: translation.z,
        })
    }
}

impl ExtractComponent for SdfInternals {
    type QueryData = &'static SdfInternals;
    type QueryFilter = Changed<SdfInternals>;
    type Out = SdfInternals;

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
