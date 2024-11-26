use crate::builtins::*;
use crate::components::initialization::RegisterSdfComponent;
use crate::groups::{SdfGroup, SdfGroupBuilderAppExt};
use crate::prelude::Annular;
use bevy::prelude::{App, Component};

pub fn plugin(app: &mut App) {
    app.sdf_group::<WorldSdf>()
        .snippet_file("embedded://cuttle/builtins/builtins.wgsl")
        .calculation("world_position", "vec2<f32>")
        .calculation("position", "vec2<f32>")
        .calculation("distance", "f32")
        .calculation("size", "f32")
        .calculation("color", "vec3<f32>")
        .calculation("prev_distance", "f32")
        .calculation("prev_color", "vec3<f32>")
        .component::<Annular>()
        .component::<Fill>()
        .component::<DistanceGradient>()
        .zst_component::<PrepareBase>()
        .zst_component::<Point>()
        .component::<Line>()
        .component::<Quad>()
        .component_with(
            RegisterSdfComponent::<GlobalTransform, GlobalTransformRender> {
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
pub struct WorldSdf;

impl SdfGroup for WorldSdf {
    // type Phase = Transparent2d;
}

#[derive(Component, Debug, Default, Clone)]
pub struct UiSdf;
