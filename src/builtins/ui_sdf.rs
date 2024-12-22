use crate::builtins::*;
use crate::components::initialization::RegisterCuttleComponent;
use crate::groups::{CuttleGroup, CuttleGroupBuilderAppExt};
use crate::prelude::Annular;
use bevy::prelude::{App, Component};
use bevy::ui::TransparentUi;

pub(super) fn plugin(app: &mut App) {
    app.sdf_group::<UiSdf>()
        .snippet_file("embedded://cuttle/builtins/builtins.wgsl")
        .calculation("world_position", "vec2<f32>")
        .calculation("position", "vec2<f32>")
        .calculation("distance", "f32")
        .calculation("color", "vec4<f32>")
        .component::<Annular>()
        .component::<Fill>()
        .zst_component::<PrepareBase>()
        .component::<super::Circle>()
        .component::<Line>()
        .component::<Quad>()
        .component_with(
            RegisterCuttleComponent::<GlobalTransform, GlobalTransformRender> {
                sort: 1100,
                ..default()
            },
        )
        .component::<Rounded>();
}

