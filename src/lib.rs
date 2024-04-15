use bevy::prelude::*;
use core::fmt::Debug;
use flag::{RenderableVariant, VariantFlag};
use render::{
    shader::{buffers::SdfVariantBuffer, variants::VariantShaderBuilder}, SdfRenderPlugin,
};
pub mod components;
pub mod flag;
mod implementations;
pub mod operations;
mod render;
mod scheduling;
pub mod prelude {
    pub use crate::components::{FillColor, GradientColor};
    pub use crate::render::shader::buffers::SdfRenderIndex;
    pub use bevy_comdf_core::prelude::*;
}

pub fn plugin(app: &mut App) {
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

trait RenderSdfComponent: Sized + Component + Debug {
    fn flag() -> VariantFlag;
    fn flag_system(mut query: Query<&mut RenderableVariant, With<Self>>) {
        query
            .iter_mut()
            .for_each(|mut variant| variant.flag |= Self::flag());
    }

    fn setup(shader: &mut VariantShaderBuilder);
    fn setup_system(mut query: Query<&mut VariantShaderBuilder, With<Self>>) {
        query.iter_mut().for_each(|mut comp| Self::setup(&mut comp));
    }

    fn prep(render: &mut SdfVariantBuffer, comp: &Self);
    fn prep_system(mut query: Query<(&mut SdfVariantBuffer, &Self)>) {
        query.iter_mut().for_each(|(mut buffer, comp)| {
            Self::prep(&mut buffer, comp);
        });
    }
}
