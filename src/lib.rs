#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::flag::Flag;
use bevy::prelude::*;
use bounding::SdfBoundingRadius;
use builtins::BuiltinsPlugin;
use components::CompPlugin;
use extensions::SdfExtensions;
use pipeline::PipelinePlugin;
use shader::ShaderPlugin;
use std::collections::BTreeMap;

mod bounding;
pub mod builtins;
mod calculations;
pub mod components;
pub mod flag;
pub mod groups;
pub mod extensions;
pub mod pipeline;
pub mod shader;
mod utils;

pub mod prelude {
    pub use crate::CuttlePlugin;
    pub use crate::bounding::BoundingSet;
    pub use crate::builtins::{groups::*, *};
    pub use crate::extensions::ExtendSdf;
    pub use crate::extensions::SdfExtensions;
}

pub struct CuttlePlugin;
impl Plugin for CuttlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CompPlugin, //Needs to be first to ensure SdfCompInfos is sorted
            BuiltinsPlugin,
            ShaderPlugin,
            PipelinePlugin,
            extensions::plugin,
            bounding::plugin,
        ));
    }
}

#[derive(Component, Debug, Default, Clone)]
#[require(Transform, Visibility, SdfExtensions, SdfBoundingRadius)]
pub struct SdfInternals {
    flag: Flag,
    indices: BTreeMap<u8, u32>,
}
