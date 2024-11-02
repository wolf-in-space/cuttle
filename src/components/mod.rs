use crate::{
    initialization::{IntoRenderData, SdfRenderData},
    Sdf,
};
use arena::IndexArena;
use bevy::{
    prelude::*,
    reflect::{StructInfo, TypeInfo},
};
use buffer::{BufferInfo, BufferPlugin};

pub mod arena;
pub mod buffer;

pub struct CompPlugin;
impl Plugin for CompPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BufferPlugin);
    }

    fn finish(&self, app: &mut App) {
        let mut infos = app.world_mut().resource_mut::<SdfCompInfos>();
        infos.sort_by_key(|s| s.order);
        let len = infos.len() as u32;
        app.world_mut().insert_resource(SdfCompCount(len));
    }
}

#[derive(Debug)]
pub struct SdfCompInfo {
    pub name: &'static str,
    pub structure: &'static StructInfo,
    pub order: u32,
    pub insert_arena: fn(&mut World, u8),
    pub buffer: BufferInfo,
}

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct SdfCompInfos(Vec<SdfCompInfo>);

#[derive(Resource, Debug, Default)]
pub struct SdfCompCount(pub u32);

impl SdfCompInfos {
    pub fn add<C: IntoRenderData<G>, G: SdfRenderData>(&mut self, order: u32) {
        let TypeInfo::Struct(structure) = G::type_info() else {
            panic!("Expected sdf component to be a Struct");
        };

        let Some(name) = G::type_ident() else {
            panic!("Expected sdf component to have a name");
        };

        self.0.push(SdfCompInfo {
            structure,
            name,
            order,
            insert_arena: IndexArena::<C>::insert,
            buffer: BufferInfo::new::<G>(),
        });
    }
}

pub(crate) fn set_flag_bit<C: Component, T, const SET: bool>(
    trigger: Trigger<T, C>,
    mut arena: ResMut<IndexArena<C>>,
    mut flags: Query<&mut Sdf>,
) {
    if let Ok(mut sdf) = flags.get_mut(trigger.entity()) {
        if SET {
            let index = arena.get();
            // println!(
            //     "{}: pos={},i={},indices={:?}",
            //     type_name::<C>(),
            //     arena.position,
            //     index,
            //     sdf.indices
            // );
            sdf.flag.set(arena.position);
            sdf.indices.insert(arena.position, index);
        } else {
            sdf.flag.unset(arena.position);
            let id = sdf.indices.remove(&arena.position).unwrap();
            arena.release(id);
        }
    }
}
