use crate::components::buffer::{BufferFns, CompBufferEntity};
use crate::components::ComponentPosition;
use crate::indices::build_set_flag_index;
use crate::internal_prelude::*;
use bevy_render::sync_world::RenderEntity;
use bevy_render::RenderApp;
use std::any::TypeId;

pub type InitObserversFn = fn(&mut App, positions: Vec<Option<ComponentPosition>>);

#[derive(Resource)]
pub struct GlobalConfigInfos {
    pub config_count: usize,
    pub component_bindings: TypeIdMap<u32>,
    pub component_observer_inits: TypeIdMap<InitObserversFn>,
    pub component_positions: Vec<TypeIdMap<ComponentPosition>>,
    pub buffer_entity: RenderEntity,
}

impl GlobalConfigInfos {
    pub(crate) fn new(app: &mut App) -> Self {
        let id = app
            .sub_app_mut(RenderApp)
            .world_mut()
            .spawn((CompBufferEntity, BufferFns::default()))
            .id();
        Self {
            config_count: 0,
            component_bindings: Default::default(),
            component_positions: Default::default(),
            component_observer_inits: Default::default(),
            buffer_entity: RenderEntity::from(id),
        }
    }

    pub fn is_registered<C: Component>(&self) -> bool {
        self.component_observer_inits
            .contains_key(&TypeId::of::<C>())
    }

    pub fn register_component<C: Component>(app: &mut App) {
        let mut global = app.world_mut().resource_mut::<GlobalConfigInfos>();
        let id = TypeId::of::<C>();
        if !global.is_registered::<C>() {
            global
                .component_observer_inits
                .insert(id, |app, positions| {
                    app.add_observer(build_set_flag_index::<true, OnAdd, C>(positions.clone()));
                    app.add_observer(build_set_flag_index::<false, OnRemove, C>(positions));
                });
        }
    }
}
