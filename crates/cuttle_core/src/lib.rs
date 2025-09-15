#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::pipeline::specialization::CuttlePipeline;
use bevy_ecs::schedule::ScheduleLabel;
use components::CompPlugin;
use internal_prelude::*;
use pipeline::PipelinePlugin;
use shader::ShaderPlugin;

pub mod bounding;
pub mod components;
pub mod configs;
pub mod debug;
pub mod extensions;
pub mod indices;
pub mod pipeline;
pub mod shader;

pub mod prelude {
    pub use crate::bounding::*;
    pub use crate::components::initialization::{Cuttle, CuttleRenderData};
    pub use crate::configs::builder::CuttleGroupBuilderAppExt;
    pub use crate::configs::CuttleConfig;
    pub use crate::extensions::ExtendedBy;
    pub use crate::extensions::Extends;
    pub use crate::pipeline::extract::CuttleZ;
    pub use crate::CuttleCorePlugin;
}

mod internal_prelude {
    pub use bevy_app::prelude::*;
    pub use bevy_derive::*;
    pub use bevy_ecs::prelude::*;
    pub use bevy_reflect::prelude::*;
    pub use bevy_render::prelude::*;
    pub use bevy_transform::prelude::*;
    pub use bevy_utils::*;
}

pub struct CuttleCorePlugin;
impl Plugin for CuttleCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ShaderPlugin,
            CompPlugin,
            PipelinePlugin,
            extensions::plugin,
            bounding::plugin,
            indices::plugin,
        ));
        use FinishCuttleSetupSet::*;
        app.configure_sets(
            FinishCuttleSetup,
            (Sort, InitPositions, CollectSnippets, LoadShaders).chain(),
        );
    }

    fn finish(&self, app: &mut App) {
        app.world_mut().run_schedule(FinishCuttleSetup);
        CuttlePipeline::init(app);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, ScheduleLabel)]
pub struct FinishCuttleSetup;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, SystemSet)]
pub enum FinishCuttleSetupSet {
    Sort,
    InitPositions,
    CollectSnippets,
    LoadShaders,
}
