use crate::groups::CuttleGroup;
use crate::CuttleFlags;
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

pub(crate) const fn build_set_flag_bit<C: Component, G: CuttleGroup, T, const SET: bool>(
    pos: u8,
) -> impl FnMut(Trigger<T, C>, ResMut<IndexArena<C>>, Query<&mut CuttleFlags>) {
    move |trigger, mut arena, mut flags| {
        if let Ok(mut flag) = flags.get_mut(trigger.entity()) {
            if SET {
                flag.flag.set(pos);
                flag.indices.insert(pos, arena.get());
            } else {
                flag.flag.unset(pos);
                let id = flag.indices.remove(&pos).unwrap();
                arena.release(id);
            }
        }
    }
}
