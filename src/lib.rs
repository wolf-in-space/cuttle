#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use builtins::BuiltinsPlugin;
use components::CompPlugin;
use pipeline::PipelinePlugin;
use shader::ShaderPlugin;
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
            indices::plugin,
        ));
    }
}

