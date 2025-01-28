#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::components::{init_component_positions, sort_component_infos};
use crate::pipeline::specialization::CuttlePipeline;
use crate::shader::load_shaders;
use bevy_ecs::system::RunSystemOnce;
use components::CompPlugin;
use configs::global::GlobalConfigInfos;
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
    pub use crate::extensions::Extension;
    pub use crate::extensions::Extensions;
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
    }

    fn finish(&self, app: &mut App) {
        let world = app.world_mut();
        world.run_system_once(sort_component_infos).unwrap();
        world.run_system_once(init_component_positions).unwrap();
        let shaders = world.run_system_once(load_shaders).unwrap();
        CuttlePipeline::init(app, shaders);

        let globals = app
            .world_mut()
            .remove_resource::<GlobalConfigInfos>()
            .unwrap();
        for (id, func) in &globals.component_observer_inits {
            let positions: Vec<_> = (0..globals.config_count)
                .map(|i| globals.component_positions[i].get(id).copied())
                .collect();

            func(app, positions);
        }
    }
}
