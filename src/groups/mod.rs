use crate::calculations::*;
use crate::components::ComponentInfos;
use crate::indices::{on_add_group_marker_initialize_indices_group_id, CuttleIndices};
use crate::pipeline::SortedCuttlePhaseItem;
use crate::shader::Snippets;
use bevy::prelude::*;
use global::{GlobalGroupInfos, InitGroupFns};
use std::marker::PhantomData;

pub mod builder;
pub mod global;

pub trait CuttleGroup: Component + Default {
    type Phase: SortedCuttlePhaseItem;
}

#[derive(Debug, Copy, Clone, Component, Reflect, Hash, Eq, PartialEq)]
#[reflect(Component)]
#[require(Calculations, Snippets, ComponentInfos)]
pub struct GroupId(pub(crate) usize);

#[derive(Resource)]
pub struct GroupStore<G> {
    pub id: usize,
    pub group: Entity,
    phantom_data: PhantomData<G>,
}
impl<G> Copy for GroupStore<G> {}
impl<G> Clone for GroupStore<G> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            group: self.group,
            phantom_data: PhantomData,
        }
    }
}

impl<G> GroupStore<G> {
    fn new(world: &mut World) -> Self {
        let mut global = world.resource_mut::<GlobalGroupInfos>();
        let id = global.group_count;
        global.group_count += 1;
        global.component_positions.push(default());
        let group = world.spawn(GroupId(id)).id();

        Self {
            id,
            group,
            phantom_data: PhantomData,
        }
    }
}

fn initialize_group<G: CuttleGroup>(app: &mut App) -> Entity {
    if let Some(store) = app.world().get_resource::<GroupStore<G>>() {
        return store.group;
    };

    if !app.world().contains_resource::<GlobalGroupInfos>() {
        app.init_resource::<InitGroupFns>();
        let infos = GlobalGroupInfos::new(app);
        app.insert_resource(infos);
    }

    app.register_required_components::<G, CuttleIndices>();
    app.world_mut()
        .register_component_hooks::<G>()
        .on_add(on_add_group_marker_initialize_indices_group_id::<G>);

    let world = app.world_mut();
    let group_id_store = GroupStore::<G>::new(world);
    world.insert_resource(group_id_store);
    group_id_store.group
}
