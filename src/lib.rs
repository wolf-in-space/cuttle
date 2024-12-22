#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bounding::CuttleBounding;
use builtins::BuiltinsPlugin;
use components::CompPlugin;
use extensions::Extensions;
use pipeline::PipelinePlugin;
use shader::ShaderPlugin;
use std::collections::BTreeMap;

mod bounding;
#[cfg(feature = "builtins")]
pub mod builtins;
mod calculations;
pub mod components;
pub mod extensions;
pub mod groups;
pub mod pipeline;
pub mod shader;

pub mod prelude {
    pub use crate::bounding::Bounding;
    #[cfg(feature = "builtins")]
    pub use crate::builtins::{self, sdf::*, ui_sdf::*, *};
    pub use crate::extensions::Extension;
    pub use crate::extensions::Extensions;
    pub use crate::CuttlePlugin;
}

pub struct CuttlePlugin;
impl Plugin for CuttlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CompPlugin, //Needs to be first to ensure SdfCompInfos is sorted
            #[cfg(feature = "builtins")]
            BuiltinsPlugin,
            ShaderPlugin,
            PipelinePlugin,
            extensions::plugin,
            bounding::plugin,
        ));
    }
}

#[derive(Component, Debug, Default, Clone)]
#[require(Transform, Visibility, Extensions, CuttleBounding)]
pub struct CuttleFlags {
    indices: BTreeMap<u8, u32>,
}
