#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod aabb;
pub mod builder;
pub mod components;
mod flag;
pub mod implementations;
pub mod operations;
pub mod pipeline;
pub mod shader;
mod utils;

pub mod prelude {
    pub use crate::builder::*;
    pub use crate::components::colors::{Fill, Gradient};
    pub use bevy_comdf_core::prelude::*;
}

use bevy::prelude::*;
use bevy::render::{ExtractSchedule, RenderApp};

use pipeline::PipelinePlugin;
use ComdfExtractSet::*;
use ComdfPostUpdateSet::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        PipelinePlugin,
        bevy_comdf_core::plugin,
        components::plugin,
        operations::plugin,
        shader::plugin,
        flag::plugin,
        implementations::plugin,
        aabb::plugin,
    ));
    app.configure_sets(
        PostUpdate,
        (
            BuildFlag,
            UpdateFlags,
            AssignBindings,
            AssignIndices,
            BuildShaders,
        )
            .chain(),
    );
    app.sub_app_mut(RenderApp)
        .configure_sets(ExtractSchedule, (PrepareExtract, Extract).chain());
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComdfExtractSet {
    PrepareExtract,
    Extract,
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComdfPostUpdateSet {
    BuildFlag,
    UpdateFlags,
    AssignBindings,
    AssignIndices,
    BuildShaders,
}
