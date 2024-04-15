use super::{draw::DrawSdf, pipeline::SdfPipeline, process::SdfInstance};
use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::{PipelineCache, SpecializedRenderPipelines},
        view::ExtractedView,
    },
    utils::FloatOrd,
};

pub fn queue_sdfs(
    sdfs: Query<(Entity, &SdfInstance)>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent2d>)>,
    sdf_pipeline: Res<SdfPipeline>,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<SdfPipeline>>,
    cache: Res<PipelineCache>,
) {
    let draw_function = draw_functions.read().id::<DrawSdf>();
    views.iter_mut().for_each(|(_view, mut render_phase)| {
        sdfs.into_iter().for_each(|(entity, sdf)| {
            let pipeline = pipelines.specialize(&cache, &sdf_pipeline, sdf.key.clone());
            render_phase.add(Transparent2d {
                sort_key: FloatOrd(0.),
                entity,
                pipeline,
                draw_function,
                batch_range: 0..1,
                dynamic_offset: None,
            });
        });
    });
}
