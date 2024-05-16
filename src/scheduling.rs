use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_render::{ExtractSchedule, Render, RenderApp, RenderSet};
use ComdfRenderSet::*;

pub fn plugin(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);

    render_app.configure_sets(ExtractSchedule, (PrepareExtract, Extract).chain());
    render_app.configure_sets(
        Render,
        (
            AfterExtract,
            BuildSdfFlags,
            BuildPipelineKeys,
            AssignBindings,
            AssignIndices,
            GatherOperationBindings,
            BuildBuffersForComponents,
            BuildBuffersForOperations,
            PrepareBuffers,
            PrepareShaderBuild,
            BuildShadersForComponents,
            BuildShadersForOperations,
            CollectShaders,
            Queue,
            PrepareBatches,
            WriteBuffers,
            BuildBindgroups,
        )
            .chain()
            .after(RenderSet::ExtractCommands)
            .before(RenderSet::Render),
    );
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComdfRenderSet {
    PrepareExtract,
    Extract,
    AfterExtract,
    BuildSdfFlags,
    BuildPipelineKeys,
    AssignBindings,
    AssignIndices,
    GatherOperationBindings,
    PrepareShaderBuild,
    BuildShadersForComponents,
    BuildShadersForOperations,
    CollectShaders,
    BuildBuffersForComponents,
    BuildBuffersForOperations,
    PrepareBuffers,
    Queue,
    PrepareBatches,
    WriteBuffers,
    BuildBindgroups,
}
