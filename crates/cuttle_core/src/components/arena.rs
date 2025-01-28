use crate::internal_prelude::*;
use bevy_utils::default;
use std::any::type_name;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Resource)]
pub(crate) struct IndexArena<C> {
    pub max: u32,
    available: Vec<u32>,
    marker: PhantomData<C>,
}

impl<C> Debug for IndexArena<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "IndexArena<{}>[max={},available={:?}]",
            type_name::<C>(),
            self.max,
            self.available
        ))
    }
}

impl<C> Default for IndexArena<C> {
    fn default() -> Self {
        Self {
            max: 0,
            available: default(),
            marker: default(),
        }
    }
}

impl<C: Component> IndexArena<C> {
    pub fn get(&mut self) -> u32 {
        self.available.pop().unwrap_or_else(|| {
            let r = self.max;
            self.max += 1;
            r
        })
    }

    pub fn release(&mut self, id: u32) {
        self.available.push(id);
    }
}
