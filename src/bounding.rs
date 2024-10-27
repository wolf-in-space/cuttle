use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SdfBoundingRadius {
    pub bounding: f32,
    compute_bounding: f32,
}

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, apply_bounding);
}

pub fn apply_bounding(mut query: Query<&mut SdfBoundingRadius>) {
    for mut sdf in &mut query {
        if sdf.bounding == sdf.compute_bounding {
            sdf.bypass_change_detection().compute_bounding = 0.;
        } else {
            sdf.bounding = sdf.compute_bounding;
            sdf.compute_bounding = 0.;
        }
    }
}

pub fn compute_aabb<C: AddToBoundingRadius>(mut query: Query<(&mut SdfBoundingRadius, &C)>) {
    for (mut sdf, c) in &mut query {
        sdf.bypass_change_detection().compute_bounding += c.compute();
    }
}

pub trait AddToBoundingRadius: Component {
    fn compute(&self) -> f32;
}
