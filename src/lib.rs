#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bounding::SdfBoundingRadius;
use builtins::BuiltinsPlugin;
use components::CompPlugin;
use flag::{Flag, FlagPlugin};
use operations::SdfExtensions;
use pipeline::PipelinePlugin;
use shader::ShaderPlugin;
use std::collections::BTreeMap;

mod bounding;
pub mod builtins;
mod calculations;
pub mod components;
pub mod flag;
pub mod initialization;
pub mod operations;
pub mod pipeline;
pub mod shader;
mod utils;

pub mod prelude {
    pub use super::*;
    pub use crate::bounding::BoundingSet;
    pub use crate::builtins::*;
    pub use crate::initialization::SdfAppExt;
    pub use crate::operations::SdfExtensions;
}

pub fn plugin(app: &mut App) {
    // app.sub_app_mut(RenderApp)
    //     .edit_schedule(Render, |schedule| {
    //         schedule.set_build_settings(ScheduleBuildSettings {
    //             ambiguity_detection: LogLevel::Warn,
    //             ..default()
    //         });
    //     });
    app.add_plugins((
        CompPlugin, //Needs to be first to ensure SdfCompInfos is sorted
        FlagPlugin,
        BuiltinsPlugin,
        ShaderPlugin,
        PipelinePlugin,
        operations::plugin,
        calculations::plugin,
        bounding::plugin,
    ));
}

#[derive(Component, Debug, Default, Clone)]
#[require(Transform, SdfExtensions, SdfBoundingRadius)]
pub struct Sdf {
    flag: Flag,
    indices: BTreeMap<u8, u32>,
}

#[derive(Component, Debug, Default, Clone)]
#[require(Sdf)]
pub struct WorldSdf;

#[derive(Component, Debug, Default, Clone)]
#[require(Sdf)]
pub struct UiSdf;
