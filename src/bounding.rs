use bevy::math::bounding::BoundingVolume;
use bevy::prelude::*;
use bevy::render::primitives::{Frustum, Sphere};
use bevy::render::{Render, RenderApp};
use bevy::render::view::{
    NoCpuCulling, NoFrustumCulling, RenderLayers, VisibilitySystems, VisibleEntities,
};
use bevy::utils::Parallel;
use crate::extensions::{Extensions};
use crate::pipeline::CuttleRenderSet;
use crate::pipeline::extract::{CombinedBounding, ExtractedBounding};

pub fn plugin(app: &mut App) {
    app.configure_sets(
        PostUpdate,
        (Bounding::Add, Bounding::Multiply, Bounding::Apply).chain(),
    )
    .add_systems(
        PostUpdate,
        (
            apply_bounding.in_set(Bounding::Apply),
            check_visibility.in_set(VisibilitySystems::CheckVisibility),
        ),
    );
    app.sub_app_mut(RenderApp).add_systems(Render, combine_bindings.in_set(CuttleRenderSet::PrepareBounds));
}

pub type InitBoundingFn = Box<dyn FnMut(&mut App) + Send + Sync>;

#[derive(Clone, Copy, Debug, Component, Default)]
pub struct CuttleBounding {
    pub bounding: f32,
    compute_bounding: f32,
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum Bounding {
    #[default]
    None,
    Add,
    Multiply,
    Apply,
}

pub fn apply_bounding(mut query: Query<&mut CuttleBounding>) {
    for mut sdf in &mut query {
        if sdf.bounding == sdf.compute_bounding {
            sdf.bypass_change_detection().compute_bounding = 0.;
        } else {
            sdf.bounding = sdf.compute_bounding;
            sdf.compute_bounding = 0.;
        }
    }
}

pub const fn make_compute_aabb_system<C: Component>(
    func: fn(&C) -> f32,
    set: Bounding,
) -> impl Fn(Query<(&mut CuttleBounding, &C)>) {
    move |mut query| {
        for (mut sdf, c) in &mut query {
            let val = func(c);
            let bounds = &mut sdf.bypass_change_detection().compute_bounding;
            match set {
                Bounding::Add => *bounds += val,
                Bounding::Multiply => *bounds *= val,
                Bounding::Apply | Bounding::None => panic!("NO"),
            }
        }
    }
}

fn combine_bindings(
    mut roots: Query<(
        &ExtractedBounding,
        &Extensions,
        &mut CombinedBounding,
    )>,
    extension_bounds: Query<&ExtractedBounding>,
) {
    for (root_bound, extensions, mut combined_bound) in &mut roots {
        combined_bound.0 = root_bound.0;

        for extension_entity in extensions.iter() {
            let bound = extension_bounds.get(*extension_entity).unwrap();
            combined_bound.0 = combined_bound.merge(&bound);
        }
    }
}

/*
fn debug_bounding(mut gizmos: Gizmos, bounds: Query<(&SdfBoundingRadius, &GlobalTransform)>) {
    for (bound, transform) in &bounds {
        gizmos.rect_2d(
            transform.translation().xy(),
            Vec2::splat(bound.bounding * 2.0),
            tailwind::BLUE_900,
        )
    }
}
*/

//TODO: Change Comments
/// System updating the visibility of entities each frame.
///
/// The system is part of the [`VisibilitySystems::CheckVisibility`] set. Each
/// frame, it updates the [`ViewVisibility`] of all entities, and for each view
/// also compute the [`VisibleEntities`] for that view.
///
/// This system needs to be run for each type of renderable entity. If you add a
/// new type of renderable entity, you'll need to add an instantiation of this
/// system to the [`VisibilitySystems::CheckVisibility`] set so that Bevy will
/// detect visibility properly for those entities.
pub fn check_visibility(
    mut thread_queues: Local<Parallel<Vec<Entity>>>,
    mut view_query: Query<(
        &mut VisibleEntities,
        &Frustum,
        Option<&RenderLayers>,
        &Camera,
        Has<NoCpuCulling>,
    )>,
    mut visible_aabb_query: Query<(
        Entity,
        &InheritedVisibility,
        &mut ViewVisibility,
        Option<&RenderLayers>,
        &CuttleBounding,
        &GlobalTransform,
        Has<NoFrustumCulling>,
    )>,
) {
    for (mut visible_entities, frustum, maybe_view_mask, camera, no_cpu_culling) in &mut view_query
    {
        if !camera.is_active {
            continue;
        }

        let view_mask = maybe_view_mask.unwrap_or_default();

        visible_aabb_query.par_iter_mut().for_each_init(
            || thread_queues.borrow_local_mut(),
            |queue, query_item| {
                let (
                    entity,
                    inherited_visibility,
                    mut view_visibility,
                    maybe_entity_mask,
                    bounding,
                    transform,
                    no_frustum_culling,
                ) = query_item;

                // Skip computing visibility for entities that are configured to be hidden.
                // ViewVisibility has already been reset in `reset_view_visibility`.
                if !inherited_visibility.get() {
                    return;
                }

                let entity_mask = maybe_entity_mask.unwrap_or_default();
                if !view_mask.intersects(entity_mask) {
                    return;
                }

                // frustum culling
                if !no_frustum_culling && !no_cpu_culling {
                    let model_sphere = Sphere {
                        center: transform.translation().into(),
                        radius: bounding.bounding,
                    };
                    if !frustum.intersects_sphere(&model_sphere, false) {
                        return;
                    }
                }

                view_visibility.set();
                queue.push(entity);
            },
        );

        visible_entities.clear::<CuttleBounding>();
        thread_queues.drain_into(visible_entities.get_mut::<CuttleBounding>());
    }
}
