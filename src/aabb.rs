use crate::{flag::SdfFlags, operations::Operations};
use bevy::prelude::*;
use bevy_comdf_core::aabb::{insert_aabb_from_sdf_size, AABB};

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, combine_aabbs.after(insert_aabb_from_sdf_size));
}

#[derive(Debug, Default, Component, Deref, DerefMut)]
pub struct CombinedAABB(pub AABB);

fn combine_aabbs(
    mut query: Query<(&mut CombinedAABB, &Operations), With<SdfFlags>>,
    aabbs: Query<&AABB>,
) {
    for (mut aabb, operations) in query.iter_mut() {
        for target in operations.keys() {
            let Ok(other) = aabbs.get(*target) else {
                error!("Operations Component held an Entry for Entity {target:?} which no longer exists / has the AABB Component");
                continue;
            };
            aabb.0 = aabb.combine(other);
        }
    }
}
