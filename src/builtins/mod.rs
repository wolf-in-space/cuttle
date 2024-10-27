use crate::{
    bounding::AddToBoundingRadius,
    initialization::{IntoRenderData, SdfAppExt},
};
use bevy::{prelude::*, render::render_resource::ShaderType};

pub struct BuiltinsPlugin;
impl Plugin for BuiltinsPlugin {
    fn build(&self, app: &mut App) {
        app.add_sdf_shader(include_str!("builtins.wgsl"));

        app.add_sdf_calculation("world_position", "vec2<f32>");
        app.add_sdf_calculation("position", "vec2<f32>");
        app.add_sdf_calculation("distance", "f32");
        app.add_sdf_calculation("color", "vec3<f32>");
        app.add_sdf_calculation("prev_distance", "f32");
        app.add_sdf_calculation("prev_color", "vec3<f32>");

        app.sdf::<Annular>().affect_aabb().register(3100);
        app.sdf::<Fill>().render_data::<FillRender>().register(5000);
        app.sdf::<Point>().affect_aabb().register(2000);
        app.sdf::<Quad>().affect_aabb().register(2200);
        app.sdf::<GlobalTransform>()
            .render_data::<GlobalTransformRender>()
            .register(1000);
        app.sdf::<Line>().affect_aabb().register(2100);
        app.sdf::<Rounded>().affect_aabb().register(3000);
        app.sdf::<Unioni>().register(10100);
        app.sdf::<Subtract>().register(10200);
        app.sdf::<Intersect>().register(10300);
        app.sdf::<SmoothUnion>().register(10400);
        app.sdf::<SmoothSubtract>().register(10500);
        app.sdf::<SmoothIntersect>().register(10600);
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Rounded {
    pub rounded: f32,
}

impl AddToBoundingRadius for Rounded {
    fn compute(&self) -> f32 {
        self.rounded
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Annular {
    pub annular: f32,
}

impl AddToBoundingRadius for Annular {
    fn compute(&self) -> f32 {
        self.annular
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Point {
    pub hi: f32,
}

impl AddToBoundingRadius for Point {
    fn compute(&self) -> f32 {
        0.
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Line {
    pub length: f32,
}

impl AddToBoundingRadius for Line {
    fn compute(&self) -> f32 {
        self.length
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Quad {
    pub half_size: Vec2,
}

impl AddToBoundingRadius for Quad {
    fn compute(&self) -> f32 {
        self.half_size.length()
    }
}

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Fill(pub Color);

#[derive(Debug, Default, ShaderType, Reflect)]
pub struct FillRender {
    pub color: Vec3,
}

impl IntoRenderData<FillRender> for Fill {
    fn into_render_data(value: &Fill) -> FillRender {
        FillRender {
            color: value.0.to_srgba().to_vec3(),
        }
    }
}

#[derive(Debug, Default, ShaderType, Reflect)]
pub struct GlobalTransformRender {
    pub transform: Mat4,
}

impl IntoRenderData<GlobalTransformRender> for GlobalTransform {
    fn into_render_data(value: &GlobalTransform) -> GlobalTransformRender {
        GlobalTransformRender {
            transform: value.compute_matrix().inverse(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Unioni {
    pub hi: u32,
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Subtract {
    pub hi: u32,
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect, ShaderType)]
#[reflect(Component)]
pub struct Intersect {
    pub hi: u32,
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
