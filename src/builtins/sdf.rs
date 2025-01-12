use crate::builtins::*;
use crate::components::initialization::RegisterCuttleComponent;
use crate::groups::{CuttleGroup, CuttleGroupBuilderAppExt};
use crate::prelude::Annular;
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::prelude::{App, Component};
use bevy::ui::TransparentUi;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((sdf_plugin::<Sdf>, sdf_plugin::<UiSdf>));
}
fn sdf_plugin<G: CuttleGroup>(app: &mut App) {
    app.cuttle_group::<G>()
        .snippet_file("embedded://cuttle/builtins/builtins.wgsl")
        .calculation("world_position", "vec2<f32>")
        .calculation("position", "vec2<f32>")
        .calculation("distance", "f32")
        .calculation("size", "f32")
        .calculation("color", "vec4<f32>")
        .calculation("prev_distance", "f32")
        .calculation("prev_color", "vec4<f32>")
        .component::<Annular>()
        .component::<Fill>()
        .component::<DistanceGradient>()
        .zst_component::<ForceFieldAlpha>()
        .zst_component::<PrepareBase>()
        .component::<Stretch>()
        .component::<Circle>()
        .component::<Line>()
        .component::<Quad>()
        .component_with(
            RegisterCuttleComponent::<GlobalTransform, GlobalTransformRender> {
                sort: TRANSFORM_POS + 100,
                ..default()
            },
        )
        .component::<Rounded>()
        .zst_component::<PrepareOperation>()
        .zst_component::<Unioni>()
        .zst_component::<Subtract>()
        .zst_component::<Intersect>()
        .zst_component::<Xor>()
        .component::<SmoothUnion>()
        .component::<SmoothSubtract>()
        .component::<SmoothIntersect>()
        .component::<SmoothXor>()
        .component::<Repetition>()
        .component::<Morph>();
}

#[derive(Component, Debug, Default, Clone)]
pub struct Sdf;

impl CuttleGroup for Sdf {
    type Phase = Transparent2d;
}

#[derive(Component, Debug, Default, Clone)]
#[require(Node)]
pub struct UiSdf;

impl CuttleGroup for UiSdf {
    type Phase = TransparentUi;
}
