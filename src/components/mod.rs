use crate::groups::SdfGroup;
use crate::SdfInternals;
use arena::IndexArena;
use bevy::prelude::*;
use buffer::BufferPlugin;

pub mod arena;
pub mod buffer;
pub mod initialization;

pub struct CompPlugin;
impl Plugin for CompPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BufferPlugin);
    }
}

pub(crate) const fn build_set_flag_bit<C: Component, G: SdfGroup, T, const SET: bool>(
    pos: u8,
) -> impl FnMut(Trigger<T, C>, ResMut<IndexArena<C>>, Query<&mut SdfInternals>) {
    move |trigger, mut arena, mut flags| {
        if let Ok(mut sdf) = flags.get_mut(trigger.entity()) {
            if SET {
                sdf.flag.set(pos);
                sdf.indices.insert(pos, arena.get());
            } else {
                sdf.flag.unset(pos);
                let id = sdf.indices.remove(&pos).unwrap();
                arena.release(id);
            }
        }
    }
}
