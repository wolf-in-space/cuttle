pub(crate) mod groups;

use crate::components::initialization::{SdfComponent, SdfRenderDataFrom, ZstSdfComponent};
use crate::prelude::BoundingSet;
use bevy::{asset::embedded_asset, prelude::*, render::render_resource::ShaderType};

pub struct BuiltinsPlugin;
impl Plugin for BuiltinsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "builtins.wgsl");
        app.add_plugins(groups::plugin);
    }
}

pub const DISTANCE_POS: u32 = 3000;

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Rounded {
    pub rounded: f32,
}

impl SdfComponent for Rounded {
    type RenderData = Self;
    const AFFECT_BOUNDS: BoundingSet = BoundingSet::Add;
    const SORT: u32 = DISTANCE_POS + 100;

    fn affect_bounds(comp: &Self) -> f32 {
        comp.rounded
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Annular {
    pub annular: f32,
}

impl SdfComponent for Annular {
    type RenderData = Self;
    const AFFECT_BOUNDS: BoundingSet = BoundingSet::Add;
    const SORT: u32 = DISTANCE_POS + 200;

    fn affect_bounds(comp: &Self) -> f32 {
        comp.annular
    }
}

pub const BASE_POS: u32 = 2000;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Point;

impl ZstSdfComponent for Point {
    const SORT: u32 = BASE_POS + 100;
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Line {
    pub length: f32,
}

impl SdfComponent for Line {
    type RenderData = Self;
    const AFFECT_BOUNDS: BoundingSet = BoundingSet::Add;
    const SORT: u32 = BASE_POS + 200;

    fn affect_bounds(comp: &Self) -> f32 {
        comp.length
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Quad {
    pub half_size: Vec2,
}

impl SdfComponent for Quad {
    type RenderData = Self;
    const AFFECT_BOUNDS: BoundingSet = BoundingSet::Add;
    const SORT: u32 = BASE_POS + 300;

    fn affect_bounds(comp: &Self) -> f32 {
        comp.half_size.length()
    }
}

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Fill(pub Srgba);

#[derive(Debug, Default, ShaderType, Reflect)]
pub struct FillRender {
    pub color: Vec3,
}

impl SdfRenderDataFrom<Fill> for FillRender {
    fn from_sdf_comp(comp: &Fill) -> Self {
        FillRender {
            color: comp.0.to_vec3(),
        }
    }
}

impl SdfComponent for Fill {
    type RenderData = FillRender;
    const SORT: u32 = 5000;
}

#[derive(Debug, Default, Clone, Component, ShaderType, Reflect)]
#[reflect(Component)]
pub struct DistanceGradient {
    pub interval: f32,
    pub color: Vec3,
}

impl SdfComponent for DistanceGradient {
    type RenderData = Self;
    const SORT: u32 = 99999;
}

#[derive(Debug, Default, ShaderType, Reflect)]
pub struct GlobalTransformRender {
    pub transform: Mat4,
}

impl SdfRenderDataFrom<GlobalTransform> for GlobalTransformRender {
    fn from_sdf_comp(comp: &GlobalTransform) -> Self {
        GlobalTransformRender {
            transform: comp.compute_matrix().inverse(),
        }
    }
}

pub const OPERATION_POS: u32 = 10000;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Unioni;

impl ZstSdfComponent for Unioni {
    const SORT: u32 = OPERATION_POS + 100;
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Subtract;

impl ZstSdfComponent for Subtract {
    const SORT: u32 = OPERATION_POS + 200;
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Intersect;

impl ZstSdfComponent for Intersect {
    const SORT: u32 = OPERATION_POS + 300;
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Xor;

impl ZstSdfComponent for Xor {
    const SORT: u32 = OPERATION_POS + 400;
}

#[derive(Debug, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct SmoothUnion {
    pub smoothness: f32,
}

impl Default for SmoothUnion {
    fn default() -> Self {
        Self { smoothness: 25. }
    }
}

impl SdfComponent for SmoothUnion {
    type RenderData = Self;
    const SORT: u32 = OPERATION_POS + 500;
}

#[derive(Debug, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct SmoothSubtract {
    pub smoothness: f32,
}

impl Default for SmoothSubtract {
    fn default() -> Self {
        Self { smoothness: 25. }
    }
}

impl SdfComponent for SmoothSubtract {
    type RenderData = Self;
    const SORT: u32 = OPERATION_POS + 600;
}

#[derive(Debug, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct SmoothIntersect {
    pub smoothness: f32,
}

impl Default for SmoothIntersect {
    fn default() -> Self {
        Self { smoothness: 25. }
    }
}

impl SdfComponent for SmoothIntersect {
    type RenderData = Self;
    const SORT: u32 = OPERATION_POS + 700;
}

#[derive(Debug, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct SmoothXor {
    pub smoothness: f32,
}

impl Default for SmoothXor {
    fn default() -> Self {
        Self { smoothness: 25. }
    }
}

impl SdfComponent for SmoothXor {
    type RenderData = Self;
    const SORT: u32 = OPERATION_POS + 800;
}

#[derive(Debug, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Repetition {
    pub scale: f32,
    pub repetitions: Vec2,
}

impl Default for Repetition {
    fn default() -> Self {
        Self {
            scale: 1.0,
            repetitions: Vec2::splat(2.),
        }
    }
}

impl SdfComponent for Repetition {
    type RenderData = Self;
    const AFFECT_BOUNDS: BoundingSet = BoundingSet::Multiply;
    const SORT: u32 = 1100;

    fn affect_bounds(comp: &Self) -> f32 {
        comp.repetitions.max_element() * comp.scale * 2.0
    }
}

#[derive(Debug, Clone, Copy, Default, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Morph {
    pub morph: f32,
}

impl SdfComponent for Morph {
    type RenderData = Self;
    const SORT: u32 = OPERATION_POS + 900;
}
