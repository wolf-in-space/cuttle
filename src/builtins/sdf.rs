use crate::bounding::Bounding;
use crate::builtins::*;
use crate::components::initialization::CuttleWrapperComponent;
use crate::groups::{builder::CuttleGroupBuilderAppExt, CuttleGroup};
use crate::shader::wgsl_struct::WgslTypeInfos;
use bevy::asset::embedded_asset;
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::prelude::{App, Component};
use bevy::render::render_resource::ShaderType;
use bevy::ui::TransparentUi;

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "builtins.wgsl");
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
    app.add_plugins(sdf_plugin::<Sdf>);
}

fn sdf_plugin<G: CuttleGroup>(app: &mut App) {
    app.cuttle_group::<G>()
        .snippet_file("embedded://cuttle/builtins/builtins.wgsl")
        .calculation("world_position", "vec2<f32>")
        .calculation("position", "vec2<f32>")
        .calculation("distance", "f32")
        .calculation("size", "f32")
        .calculation("prev_distance", "f32")
        .calculation("prev_color", "vec4<f32>")
        .register_component_manual::<Sdf, f32>(SdfOrder::Result, None, None, Some(255))
        .marker_component::<PrepareOperation>(SdfOrder::Prepare)
        .marker_component::<PrepareBase>(SdfOrder::Prepare)
        .wrapper_component::<Annular>(SdfOrder::Distance)
        .affect_bounds(Bounding::Add, |&Annular(a)| a)
        .wrapper_component::<Fill>(SdfOrder::Distance)
        .component::<DistanceGradient>(SdfOrder::Last)
        .marker_component::<ForceFieldAlpha>(SdfOrder::Alpha)
        .wrapper_component::<Stretch>(u32::from(SdfOrder::Translation) + 100)
        .affect_bounds(Bounding::Multiply, |&Stretch(s)| (s.length() + 1.) * 20.)
        .wrapper_component::<Circle>(SdfOrder::Base)
        .affect_bounds(Bounding::Add, |&Circle(c)| c)
        .wrapper_component::<Line>(SdfOrder::Base)
        .affect_bounds(Bounding::Add, |&Line(l)| l)
        .wrapper_component::<Quad>(SdfOrder::Base)
        .affect_bounds(Bounding::Add, |&Quad(q)| q.length())
        .wrapper_component::<Rounded>(SdfOrder::Distance)
        .affect_bounds(Bounding::Add, |&Rounded(r)| r)
        .marker_component::<Unioni>(SdfOrder::Operations)
        .marker_component::<Subtract>(SdfOrder::Operations)
        .marker_component::<Intersect>(SdfOrder::Operations)
        .marker_component::<Xor>(SdfOrder::Operations)
        .wrapper_component::<SmoothUnion>(SdfOrder::Operations)
        .wrapper_component::<SmoothSubtract>(SdfOrder::Operations)
        .wrapper_component::<SmoothIntersect>(SdfOrder::Operations)
        .wrapper_component::<SmoothXor>(SdfOrder::Operations)
        .component::<Repetition>(SdfOrder::Operations)
        .wrapper_component::<Morph>(SdfOrder::Operations)
        .register_component_manual(
            SdfOrder::Translation,
            Some(WgslTypeInfos::wgsl_type_for_builtin::<Mat4>),
            Some(|g: &GlobalTransform| g.compute_matrix().inverse()),
            None,
        );
}

#[derive(Component, Debug, Default, Clone, Reflect)]
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

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct PrepareBase;

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Rounded(pub f32);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Annular(pub f32);

#[derive(Debug, Default, Copy, Clone, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
#[require(PrepareBase)]
pub struct Circle(pub f32);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
#[require(PrepareBase)]
pub struct Line(pub f32);

#[derive(Debug, Default, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
#[require(PrepareBase)]
pub struct Quad(pub Vec2);

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Fill(pub Srgba);

impl CuttleWrapperComponent for Fill {
    type RenderData = Vec4;
    fn to_render_data(&self) -> Self::RenderData {
        self.0.to_vec4()
    }
}

#[derive(Debug, Default, Clone, Component, ShaderType, Reflect)]
#[reflect(Component)]
pub struct DistanceGradient {
    pub interval: f32,
    pub color: Vec4,
}

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct ForceFieldAlpha;

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct PrepareOperation;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Unioni;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Subtract;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Intersect;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Xor;

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct SmoothUnion(pub f32);

impl Default for SmoothUnion {
    fn default() -> Self {
        Self(25.)
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct SmoothSubtract(pub f32);

impl Default for SmoothSubtract {
    fn default() -> Self {
        Self(25.)
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct SmoothIntersect(pub f32);

impl Default for SmoothIntersect {
    fn default() -> Self {
        Self(25.)
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct SmoothXor(pub f32);

impl Default for SmoothXor {
    fn default() -> Self {
        Self(25.)
    }
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

#[derive(Debug, Clone, Copy, Default, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
#[require(PrepareOperation)]
pub struct Morph(pub f32);

#[derive(Debug, Clone, Copy, Default, Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Stretch(pub Vec2);
