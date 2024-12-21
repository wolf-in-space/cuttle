use crate::builtins::*;
use crate::components::initialization::RegisterCuttleComponent;
use crate::groups::{CuttleGroup, CuttleGroupBuilderAppExt};
use crate::prelude::Annular;
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::prelude::{App, Component};

pub(super) fn plugin(app: &mut App) {
    app.sdf_group::<Sdf>()
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
        .component::<Circle>()
        .component::<Line>()
        .component::<Quad>()
        .component_with(
            RegisterCuttleComponent::<GlobalTransform, GlobalTransformRender> {
                sort: 1100,
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
