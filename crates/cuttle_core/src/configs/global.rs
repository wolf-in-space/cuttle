use crate::components::buffer::{BufferFns, CompBufferEntity};
use crate::internal_prelude::*;
use bevy_render::sync_world::RenderEntity;
use bevy_render::RenderApp;

#[derive(Resource)]
pub struct GlobalConfigInfos {
    pub config_count: usize,
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
            buffer_entity: RenderEntity::from(id),
        }
    }
}
