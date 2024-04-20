use self::{
    draw::DrawSdf,
    pipeline::{SdfPipeline, SdfSpecializationData},
    queue::queue_sdfs,
    shader::loading::SdfShaderRegister,
};
use crate::scheduling::ComdfRenderSet::*;
use aery::Aery;
use bevy_app::{App, Plugin};
use bevy_core_pipeline::core_2d::Transparent2d;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_render::{
    render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, Render, RenderApp,
};

mod draw;
pub mod extract;
pub mod pipeline;
mod process;
mod queue;
pub mod shader;

pub struct SdfRenderPlugin;
impl Plugin for SdfRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            shader::loading::plugin,
            shader::buffers::plugin,
            extract::plugin,
            process::plugin,
        ));
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_plugins(Aery);
        render_app
            .init_resource::<SpecializedRenderPipelines<SdfPipeline>>()
            .init_resource::<SdfShaderRegister>()
            .add_event::<SdfSpecializationData>()
            .add_render_command::<Transparent2d, DrawSdf>()
            .add_systems(Render, (queue_sdfs.in_set(Queue),));
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<SdfPipeline>();
        }
    }
}
