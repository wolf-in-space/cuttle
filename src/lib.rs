#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::components::{init_component_positions, sort_component_infos};
use crate::pipeline::specialization::CuttlePipeline;
use crate::shader::load_shaders;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use builtins::BuiltinsPlugin;
use components::CompPlugin;
use groups::global::GlobalGroupInfos;
use pipeline::PipelinePlugin;
use shader::ShaderPlugin;

pub mod bounding;
#[cfg(feature = "builtins")]
pub mod builtins;
mod calculations;
pub mod components;
pub mod debug;
pub mod extensions;
pub mod groups;
pub mod indices;
pub mod pipeline;
pub mod shader;

pub mod prelude {
    pub use crate::bounding::Bounding;
    #[cfg(feature = "builtins")]
    pub use crate::builtins::{self, sdf::*, *};
    pub use crate::components::initialization::{Cuttle, CuttleRenderData};
    pub use crate::extensions::Extension;
    pub use crate::extensions::Extensions;
    pub use crate::CuttlePlugin;
}

pub struct CuttlePlugin;
impl Plugin for CuttlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ShaderPlugin,
            CompPlugin,
            #[cfg(feature = "builtins")]
            BuiltinsPlugin,
            PipelinePlugin,
            extensions::plugin,
            bounding::plugin,
            indices::plugin,
            calculations::plugin,
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
            .remove_resource::<GlobalGroupInfos>()
            .unwrap();
        for (id, func) in &globals.component_observer_inits {
            let positions: Vec<_> = (0..globals.group_count)
                .map(|i| globals.component_positions[i].get(id).copied())
                .collect();

            func(app, positions);
        }
    }
}
