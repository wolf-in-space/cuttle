use bevy_app::{App, Plugin};
use bevy_asset::embedded_asset;
use bevy_color::{ColorToComponents, Srgba};
use bevy_core_pipeline::core_2d::Transparent2d;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::Component;
use bevy_ecs::prelude::ReflectComponent;
use bevy_math::prelude::*;
use bevy_reflect::Reflect;
use bevy_render::render_resource::ShaderType;
use bevy_transform::prelude::GlobalTransform;
use cuttle_core::prelude::{Bounding, CuttleConfig, CuttleGroupBuilderAppExt};
use cuttle_macros::Cuttle;

pub struct SdfPlugin;
impl Plugin for SdfPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<(
            Sdf,
            DistanceGradient,
            PrepareBase,
            Annular,
            Circle,
            Line,
            Quad,
            Fill,
            ForceFieldAlpha,
            Stretch,
            Rounded,
        )>()
        .register_type::<(
            PrepareOperation,
            Unioni,
            Subtract,
            Intersect,
            Xor,
            SmoothUnion,
            SmoothSubtract,
            SmoothIntersect,
            SmoothXor,
            Repetition,
            Morph,
        )>();

        embedded_asset!(app, "sdf.wgsl");

        app.cuttle_config::<Sdf>()
            .snippet_file("embedded://cuttle_sdf/sdf.wgsl")
            .variable("world_position", "vec2<f32>")
            .variable("position", "vec2<f32>")
            .variable("distance", "f32")
            .variable("size", "f32")
            .variable("prev_distance", "f32")
            .variable("prev_color", "vec4<f32>")
            .components::<(
                Sdf,
                DistanceGradient,
                PrepareBase,
                Annular,
                Circle,
                Line,
                Quad,
                Fill,
                ForceFieldAlpha,
                Stretch,
                Rounded,
            )>()
            .components::<(
                PrepareOperation,
                Unioni,
                Subtract,
                Intersect,
                Xor,
                SmoothUnion,
                SmoothSubtract,
                SmoothIntersect,
                SmoothXor,
                Repetition,
                Morph,
            )>()
            .affect_bounds(Bounding::Add, |&Annular(a)| a)
            .affect_bounds(Bounding::Multiply, |&Stretch(s)| (s.length() + 1.) * 20.)
            .affect_bounds(Bounding::Add, |&Circle(c)| c)
            .affect_bounds(Bounding::Add, |&Line(l)| l)
            .affect_bounds(Bounding::Add, |&Quad(q)| q.length())
            .affect_bounds(Bounding::Add, |&Rounded(r)| r);

        app.cuttle_config::<Sdf>()
            .component_manual::<GlobalTransform>()
            .name("GlobalTransform")
            .sort(SdfOrder::Translation)
            .render_data_manual(tranfsorm_to_mat4);
    }
}

fn tranfsorm_to_mat4(t: &GlobalTransform) -> Mat4 {
    t.compute_matrix()
}

#[derive(Component, Debug, Default, Clone, Reflect, Cuttle)]
#[cuttle(extension_index_override(255u8))]
#[cuttle(sort(SdfOrder::Result))]
pub struct Sdf;

impl CuttleConfig for Sdf {
    type Phase = Transparent2d;
}

#[derive(Copy, Clone)]
pub enum SdfOrder {
    Prepare = 1000,
    Translation = 2000,
    Base = 3000,
    Distance = 4000,
    Color = 5000,
    Alpha = 6000,
    Operations = 7000,
    Last = 8000,
    Result = 9999,
}

impl From<SdfOrder> for u32 {
    fn from(value: SdfOrder) -> Self {
        value as u32
    }
}

#[derive(Debug, Component, Reflect, Default, Cuttle)]
#[cuttle(sort(SdfOrder::Prepare))]
#[reflect(Component)]
pub struct PrepareBase;

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Distance))]
#[reflect(Component)]
pub struct Rounded(pub f32);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Distance))]
#[reflect(Component)]
pub struct Annular(pub f32);

#[derive(Debug, Default, Copy, Clone, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Base))]
#[reflect(Component)]
#[require(PrepareBase)]
pub struct Circle(pub f32);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Base))]
#[reflect(Component)]
#[require(PrepareBase)]
pub struct Line(pub f32);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Base))]
#[reflect(Component)]
#[require(PrepareBase)]
pub struct Quad(pub Vec2);

#[derive(Debug, Default, Clone, Component, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Color))]
#[cuttle(render_data(Vec4))]
#[reflect(Component)]
pub struct Fill(pub Srgba);

impl From<&Fill> for Vec4 {
    fn from(value: &Fill) -> Self {
        value.0.to_vec4()
    }
}

#[derive(Debug, Default, Clone, Component, ShaderType, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Last))]
#[reflect(Component)]
pub struct DistanceGradient {
    pub interval: f32,
    pub color: Vec4,
}

#[derive(Debug, Default, Clone, Component, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Alpha))]
#[reflect(Component)]
pub struct ForceFieldAlpha;

#[derive(Debug, Component, Reflect, Default, Cuttle)]
#[cuttle(sort(SdfOrder::Prepare))]
#[reflect(Component)]
pub struct PrepareOperation;

#[derive(Debug, Default, Component, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Unioni;

#[derive(Debug, Default, Component, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Subtract;

#[derive(Debug, Default, Component, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Intersect;

#[derive(Debug, Default, Component, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Xor;

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct SmoothUnion(pub f32);

impl Default for SmoothUnion {
    fn default() -> Self {
        Self(25.)
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct SmoothSubtract(pub f32);

impl Default for SmoothSubtract {
    fn default() -> Self {
        Self(25.)
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct SmoothIntersect(pub f32);

impl Default for SmoothIntersect {
    fn default() -> Self {
        Self(25.)
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct SmoothXor(pub f32);

impl Default for SmoothXor {
    fn default() -> Self {
        Self(25.)
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, ShaderType, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
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

#[derive(Debug, Clone, Copy, Default, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Operations))]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Morph(pub f32);

#[derive(Debug, Clone, Copy, Default, Component, Reflect, Deref, DerefMut, Cuttle)]
#[cuttle(sort(SdfOrder::Translation))]
#[reflect(Component)]
pub struct Stretch(pub Vec2);
