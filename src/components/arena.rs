use bevy::prelude::{Resource, World};
use std::any::type_name;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Resource)]
pub(crate) struct IndexArena<C> {
    pub position: u8,
    pub max: u32,
    available: Vec<u32>,
    marker: PhantomData<C>,
}

impl<C> Debug for IndexArena<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "IndexArena<{}>[pos={},max={},avalible={:?}]",
            type_name::<C>(),
            self.position,
            self.max,
            self.available
        ))
    }
}

impl<C: Send + Sync + 'static> IndexArena<C> {
    fn new(position: u8) -> Self {
        Self {
            position,
            max: 0,
            available: vec![],
            marker: PhantomData,
        }
    }

    pub fn insert(world: &mut World, index: u8) {
        world.insert_resource(Self::new(index))
    }

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
