use bevy::{color::palettes::tailwind, input::common_conditions::input_pressed, prelude::*};

pub fn plugin(app: &mut App) {
    app.configure_sets(
        PostUpdate,
        (BoundingSet::Add, BoundingSet::Mult, BoundingSet::Apply).chain(),
    )
    .add_systems(PostUpdate, apply_bounding.in_set(BoundingSet::Apply))
    .add_systems(
        PostUpdate,
        debug_bounding
            .after(BoundingSet::Apply)
            .run_if(input_pressed(KeyCode::KeyB)),
    );
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SdfBoundingRadius {
    pub bounding: f32,
    compute_bounding: f32,
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum BoundingSet {
    Add,
    Mult,
    Apply,
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

pub const fn make_compute_aabb_sytem<C: Component>(
    func: fn(&C) -> f32,
    set: BoundingSet,
) -> impl Fn(Query<(&mut SdfBoundingRadius, &C)>) {
    move |mut query| {
        for (mut sdf, c) in &mut query {
            let val = func(c);
            let bounds = &mut sdf.bypass_change_detection().compute_bounding;
            match set {
                BoundingSet::Add => *bounds += val,
                BoundingSet::Mult => *bounds *= val,
                BoundingSet::Apply => panic!("NO"),
            }
        }
    }
}

fn debug_bounding(mut gizmos: Gizmos, bounds: Query<(&SdfBoundingRadius, &GlobalTransform)>) {
    for (bound, transform) in &bounds {
        gizmos.rect_2d(
            transform.translation().xy(),
            Vec2::splat(bound.bounding * 2.0),
            tailwind::BLUE_900,
        )
    }
}
