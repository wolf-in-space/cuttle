#![allow(clippy::type_complexity)]

pub mod components;
pub mod flag;
mod implementations;
pub mod operations;
mod render;
mod scheduling;

use bevy_app::prelude::*;
use bevy_comdf_core::prepare::{add_a_if_with_b_and_without_a, Sdf};
use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::prelude::*;
use core::fmt::Debug;
use render::SdfRenderPlugin;

pub mod prelude {
    pub use crate::components::{Fill, Gradient};
    pub use crate::render::shader::buffers::SdfStorageIndex;
    pub use bevy_comdf_core::prelude::*;
}

pub fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, add_a_if_with_b_and_without_a::<Sdf, RenderSdf>);
    app.add_plugins((
        SdfRenderPlugin,
        bevy_comdf_core::plugin,
        flag::plugin,
        components::plugin,
        operations::plugin,
        implementations::plugin,
        scheduling::plugin,
    ));
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct RenderSdf;
