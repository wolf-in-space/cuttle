use crate::extensions::Extensions;
use bevy::math::bounding::{BoundingCircle, BoundingVolume};
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::primitives::{Frustum, Sphere};
use bevy::render::view::{
    NoCpuCulling, NoFrustumCulling, RenderLayers, VisibilitySystems, VisibleEntities,
};
use bevy::utils::Parallel;

pub fn plugin(app: &mut App) {
    app.register_type::<BoundingRadius>()
        .register_type::<GlobalBoundingCircle>()
        .configure_sets(PostUpdate, (Bounding::Add, Bounding::Multiply).chain())
        .configure_sets(PostUpdate, ComputeGlobalBounding.before(check_visibility))
        .add_systems(
            PostUpdate,
            (
                compute_global_bounding_circles.in_set(ComputeGlobalBounding),
                check_visibility.in_set(VisibilitySystems::CheckVisibility),
            ),
        );
}

#[derive(Debug, SystemSet, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub struct ComputeGlobalBounding;

#[derive(Clone, Copy, Debug, Component, Default, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct BoundingRadius(pub f32);

#[derive(Clone, Copy, Debug, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct GlobalBoundingCircle(BoundingCircle);

impl Default for GlobalBoundingCircle {
    fn default() -> Self {
        Self(BoundingCircle::new(Vec2::ZERO, 0.0))
    }
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Bounding {
    Add,
    Multiply,
}

#[inline]
pub const fn make_compute_aabb_system<C: Component>(
    func: fn(&C) -> f32,
    set: Bounding,
) -> impl Fn(Query<(&mut BoundingRadius, &C)>) {
    move |mut query| {
        for (mut bounding, c) in &mut query {
            let val = func(c);
            match set {
                Bounding::Add => **bounding += val,
                Bounding::Multiply => **bounding *= val,
            }
        }
    }
}

fn compute_global_bounding_circles(
    mut roots: Query<(
        &Transform,
        &mut BoundingRadius,
        &Extensions,
        &mut GlobalBoundingCircle,
    )>,
    mut extension_bounds: Query<(&Transform, &mut BoundingRadius), Without<GlobalBoundingCircle>>,
) {
    for (transform, mut radius, extensions, mut bounding) in &mut roots {
        **bounding = BoundingCircle::new(transform.translation.xy(), **radius);
        **radius = default();

        for extension_entity in extensions.iter() {
            if let Ok((transform, mut radius)) = extension_bounds.get_mut(*extension_entity) {
                **bounding =
                    bounding.merge(&BoundingCircle::new(transform.translation.xy(), **radius));
                **radius = default();
            }
        }
    }
}

// TODO: Change Comments
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
        &GlobalBoundingCircle,
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
                        center: Vec3A::from(bounding.center.extend(0.)),
                        radius: bounding.circle.radius,
                    };
                    if !frustum.intersects_sphere(&model_sphere, false) {
                        return;
                    }
                }

                view_visibility.set();
                queue.push(entity);
            },
        );

        visible_entities.clear::<BoundingRadius>();
        thread_queues.drain_into(visible_entities.get_mut::<BoundingRadius>());
    }
}
