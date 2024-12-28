#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use builtins::BuiltinsPlugin;
use components::CompPlugin;
use pipeline::PipelinePlugin;
use shader::ShaderPlugin;
use crate::groups::{GlobalGroupInfos, InitGroupFns};

mod bounding;
#[cfg(feature = "builtins")]
pub mod builtins;
mod calculations;
pub mod components;
pub mod extensions;
pub mod groups;
pub mod pipeline;
pub mod shader;
mod indices;

pub mod prelude {
    pub use crate::bounding::Bounding;
    #[cfg(feature = "builtins")]
    pub use crate::builtins::{self, sdf::*, *};
    pub use crate::extensions::Extension;
    pub use crate::extensions::Extensions;
    pub use crate::CuttlePlugin;
}

pub struct CuttlePlugin;
impl Plugin for CuttlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ShaderPlugin,
            CompPlugin, //Needs to be first to ensure SdfCompInfos is sorted
            #[cfg(feature = "builtins")]
            BuiltinsPlugin,
            PipelinePlugin,
            extensions::plugin,
            bounding::plugin,
        ));
    }

    fn finish(&self, app: &mut App) {
        let init_groups = app.world_mut().remove_resource::<InitGroupFns>().unwrap();

        for init_group in init_groups.iter() {
            init_group(app);
        }

        let globals = app.world_mut().remove_resource::<GlobalGroupInfos>().unwrap();

        for (id, func) in &globals.component_observer_inits {
            let positions: Vec<_> = (0..globals.group_count)
                .into_iter()
                .map(|i| globals.component_positions[i].get(id).copied())
                .collect();

            if let Some(init_extract) = globals.component_extract_inits.get(id) {
                init_extract(app, positions.clone())
            }

            func(app, positions);
        }
    }
}

