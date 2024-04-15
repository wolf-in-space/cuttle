use self::{
    draw::DrawSdf,
    extract::{
        extract_loaded_specialization_data, extract_render_sdfs, extract_sdf_variants,
        extract_variant_data, ExtractedRenderSdfs, ExtractedSdfVariants,
    },
    pipeline::{receive_specialization, SdfPipeline, SdfSpecializationData},
    process::{
        create_bind_groups_for_new_keys, prepare_view_bind_groups, process_render_sdfs,
        process_sdf_variants, write_buffers, SdfBindGroups,
    },
    queue::queue_sdfs,
    shader::loading::SdfShaderRegister,
};
use bevy::{
    app::{App, Plugin},
    core_pipeline::core_2d::Transparent2d,
    ecs::schedule::IntoSystemConfigs,
    render::{
        render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines,
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

mod draw;
mod extract;
pub mod pipeline;
mod process;
mod queue;
pub mod shader;

pub struct SdfRenderPlugin;
impl Plugin for SdfRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((shader::loading::plugin, shader::buffers::plugin));
        app.init_resource::<SdfShaderRegister>();
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<SpecializedRenderPipelines<SdfPipeline>>()
                .init_resource::<SdfBindGroups>()
                .init_resource::<ExtractedSdfVariants>()
                .init_resource::<ExtractedRenderSdfs>()
                .add_event::<SdfSpecializationData>()
                .add_render_command::<Transparent2d, DrawSdf>()
                .add_systems(
                    ExtractSchedule,
                    (
                        extract_sdf_variants,
                        extract_render_sdfs,
                        extract_variant_data,
                        extract_loaded_specialization_data,
                    ),
                )
                .add_systems(
                    Render,
                    (
                        (
                            process_sdf_variants,
                            process_render_sdfs,
                            receive_specialization,
                        )
                            .in_set(RenderSet::PrepareAssets),
                        queue_sdfs.in_set(RenderSet::Queue),
                        prepare_view_bind_groups.in_set(RenderSet::PrepareBindGroups),
                        (write_buffers, create_bind_groups_for_new_keys)
                            .chain()
                            .in_set(RenderSet::Prepare),
                    ),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<SdfPipeline>();
        }
    }
}
