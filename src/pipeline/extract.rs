use super::RenderPhase;
use crate::{
    bounding::SdfBoundingRadius,
    components::{arena::IndexArena, buffer::CompBuffer},
    flag::Flag,
    initialization::{IntoRenderData, SdfRenderData},
    operations::SdfExtensions,
    Sdf, UiSdf, WorldSdf,
};
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    math::bounding::BoundingCircle,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        sync_component::SyncComponentPlugin,
        sync_world::RenderEntity,
        Extract, RenderApp,
    },
    ui::TransparentUi,
};
use std::{any::type_name, marker::PhantomData};
use std::{collections::BTreeMap, fmt::Debug};

pub fn plugin(app: &mut App) {
    app.add_plugins((
        SyncComponentPlugin::<SdfExtensions>::default(),
        ExtractComponentPlugin::<Sdf>::default(),
        ExtractComponentPlugin::<WorldSdf>::default(),
        ExtractComponentPlugin::<UiSdf>::default(),
    ))
    .sub_app_mut(RenderApp)
    .add_systems(ExtractSchedule, extract_sdf_extensions);
}

pub(crate) fn extract_sdf_comp<C: Component + IntoRenderData<G>, G: SdfRenderData>(
    mut buffer: Single<&mut CompBuffer<G>>,
    arena: Extract<Res<IndexArena<C>>>,
    comps: Extract<Query<(&Sdf, &C), Changed<C>>>,
) {
    let buffer = buffer.get_mut();
    buffer.resize_with(arena.max as usize, || G::default());

    for (sdf, comp) in &comps {
        let index = *sdf.indices.get(&arena.position).unwrap() as usize;
        let elem = buffer.get_mut(index).unwrap();
        *elem = C::into_render_data(comp);
    }

    trace_once!(
        "SdfIndexArena for {}: {:#?}",
        type_name::<G>(),
        arena.as_ref()
    );
    trace_once!("SdfBuffer for {}: {:#?}", type_name::<G>(), &buffer);
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
pub struct ExtractedSdf {
    pub flag: Flag,
    pub indices: BTreeMap<u8, u32>,
    pub bounding: BoundingCircle,
}

#[derive(Component, Debug)]
pub struct ExtractedRenderSdf {
    pub sort: f32,
    pub op_start_index: u32,
    pub op_count: u32,
    pub final_bounds: BoundingCircle,
}

#[derive(Component)]
pub struct PipelineMarker<P: RenderPhase>(PhantomData<P>);

impl<P: RenderPhase> PipelineMarker<P> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl ExtractComponent for WorldSdf {
    type QueryData = &'static GlobalTransform;
    type QueryFilter = (With<WorldSdf>, Changed<GlobalTransform>);
    type Out = (ExtractedRenderSdf, PipelineMarker<Transparent2d>);

    fn extract_component(&transform: &GlobalTransform) -> Option<Self::Out> {
        let translation = transform.translation();
        Some((
            ExtractedRenderSdf {
                sort: translation.z,
                op_count: 0,
                op_start_index: 0,
                final_bounds: BoundingCircle::new(Vec2::ZERO, 0.),
            },
            PipelineMarker::new(),
        ))
    }
}

impl ExtractComponent for UiSdf {
    type QueryData = &'static GlobalTransform;
    type QueryFilter = With<UiSdf>;
    type Out = (ExtractedRenderSdf, PipelineMarker<TransparentUi>);

    fn extract_component(&transform: &GlobalTransform) -> Option<Self::Out> {
        let translation = transform.translation();
        Some((
            ExtractedRenderSdf {
                sort: translation.z,
                op_count: 0,
                op_start_index: 0,
                final_bounds: BoundingCircle::new(Vec2::ZERO, 0.),
            },
            PipelineMarker::new(),
        ))
    }
}

impl ExtractComponent for Sdf {
    type QueryData = (
        &'static Sdf,
        &'static SdfBoundingRadius,
        &'static GlobalTransform,
    );
    type QueryFilter = Or<(Changed<Sdf>, Changed<SdfBoundingRadius>)>;
    type Out = ExtractedSdf;

    fn extract_component(
        (sdf, bounding, t): (&Sdf, &SdfBoundingRadius, &GlobalTransform),
    ) -> Option<Self::Out> {
        Some(ExtractedSdf {
            flag: sdf.flag,
            bounding: BoundingCircle::new(t.translation().xy(), bounding.bounding),
            indices: sdf.indices.clone(),
        })
    }
}
