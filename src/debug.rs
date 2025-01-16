use crate::bounding::{BoundingRadius, ComputeGlobalBounding, GlobalBoundingCircle};
use bevy::app::App;
use bevy::prelude::{
    Gizmos, GlobalTransform, IntoSystemConfigs, Plugin, PostUpdate, Query, Res, Resource, Srgba,
    Vec3Swizzles,
};

#[derive(Resource, Default, Clone, Copy)]
pub struct CuttleDebugPlugin {
    pub global_bounds: bool,
    pub local_bounds: bool,
}

impl Plugin for CuttleDebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(*self);
        app.add_systems(
            PostUpdate,
            (
                combined_bounds.run_if(|debug: Res<CuttleDebugPlugin>| debug.global_bounds),
                local_bounds.run_if(|debug: Res<CuttleDebugPlugin>| debug.local_bounds),
            )
                .after(ComputeGlobalBounding),
        );
    }
}

fn combined_bounds(mut gizmos: Gizmos, query: Query<&GlobalBoundingCircle>) {
    for bounding in &query {
        gizmos.circle_2d(bounding.center, bounding.circle.radius, Srgba::RED);
    }
}

fn local_bounds(mut gizmos: Gizmos, query: Query<(&GlobalTransform, &BoundingRadius)>) {
    for (transform, radius) in &query {
        gizmos.circle_2d(transform.translation().xy(), **radius, Srgba::GREEN);
    }
}
