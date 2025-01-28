use bevy_app::{App, Plugin};
use bevy_asset::embedded_asset;
use bevy_color::{ColorToComponents, Srgba};
use bevy_core_pipeline::core_2d::Transparent2d;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::Component;
use bevy_ecs::prelude::ReflectComponent;
use bevy_math::{Mat4, Vec2, Vec4};
use bevy_reflect::Reflect;
use bevy_render::render_resource::ShaderType;
use bevy_transform::prelude::GlobalTransform;
use cuttle_core::components::initialization::init_render_data;
use cuttle_core::prelude::{Bounding, CuttleConfig, CuttleGroupBuilderAppExt};
use cuttle_core::shader::wgsl_struct::WgslTypeInfos;
use cuttle_core::shader::ToRenderData;
use cuttle_macros::Cuttle;

pub struct SdfPlugin;
impl Plugin for SdfPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Sdf>()
            .register_type::<Rounded>()
            .register_type::<Annular>()
            .register_type::<PrepareBase>()
            .register_type::<Circle>()
            .register_type::<Line>()
            .register_type::<Quad>()
            .register_type::<Fill>()
            .register_type::<DistanceGradient>()
            .register_type::<ForceFieldAlpha>()
            .register_type::<PrepareOperation>()
            .register_type::<Unioni>()
            .register_type::<SmoothUnion>()
            .register_type::<Subtract>()
            .register_type::<SmoothSubtract>()
            .register_type::<Intersect>()
            .register_type::<SmoothIntersect>()
            .register_type::<Xor>()
            .register_type::<Morph>()
            .register_type::<Repetition>();

        embedded_asset!(app, "sdf.wgsl");

        app.cuttle_config::<Sdf>()
            .snippet_file("embedded://cuttle_sdf/sdf.wgsl")
            .calculation("world_position", "vec2<f32>")
            .calculation("position", "vec2<f32>")
            .calculation("distance", "f32")
            .calculation("size", "f32")
            .calculation("prev_distance", "f32")
            .calculation("prev_color", "vec4<f32>")
            .component::<DistanceGradient>()
            .component::<Sdf>()
            .component::<PrepareOperation>()
            .component::<PrepareBase>()
            .component::<Annular>()
            .affect_bounds(Bounding::Add, |&Annular(a)| a)
            .component::<Fill>()
            .component::<ForceFieldAlpha>()
            .component::<Stretch>()
            .affect_bounds(Bounding::Multiply, |&Stretch(s)| (s.length() + 1.) * 20.)
            .component::<Circle>()
            .affect_bounds(Bounding::Add, |&Circle(c)| c)
            .component::<Line>()
            .affect_bounds(Bounding::Add, |&Line(l)| l)
            .component::<Quad>()
            .affect_bounds(Bounding::Add, |&Quad(q)| q.length())
            .component::<Rounded>()
            .affect_bounds(Bounding::Add, |&Rounded(r)| r)
            .component::<Unioni>()
            .component::<Subtract>()
            .component::<Intersect>()
            .component::<Xor>()
            .component::<SmoothUnion>()
            .component::<SmoothSubtract>()
            .component::<SmoothIntersect>()
            .component::<SmoothXor>()
            .component::<Repetition>()
            .component::<Morph>();

        let global_transform_binding =
            init_render_data(app, |g: &GlobalTransform| g.compute_matrix().inverse());

        app.cuttle_config::<Sdf>()
            .register_component_manual::<GlobalTransform>(
                SdfOrder::Translation,
                Some(ToRenderData {
                    binding: global_transform_binding,
                    to_wgsl: WgslTypeInfos::wgsl_type_for_builtin::<Mat4>,
                }),
                None,
            );
    }
}

#[derive(Component, Debug, Default, Clone, Reflect, Cuttle)]
#[cuttle(sort(SdfOrder::Result))]
#[cuttle(extension_index_override(255u8))]
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

impl From<Fill> for Vec4 {
    fn from(value: Fill) -> Self {
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
