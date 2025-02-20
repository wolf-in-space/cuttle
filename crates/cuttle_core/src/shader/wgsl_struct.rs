use crate::internal_prelude::*;
use bevy_math::{Mat2, Mat4, Vec2, Vec3, Vec4};
use bevy_reflect::{TypeInfo, Typed};
use std::any::{type_name, TypeId};
use std::fmt::Write;

pub fn plugin(app: &mut App) {
    app.register_wgsl_type::<f32>("f32");
    app.register_wgsl_type::<u32>("u32");
    app.register_wgsl_type::<i32>("i32");
    app.register_wgsl_type::<Vec2>("vec2<f32>");
    app.register_wgsl_type::<Vec3>("vec3<f32>");
    app.register_wgsl_type::<Vec4>("vec4<f32>");
    app.register_wgsl_type::<Mat2>("mat2x2<f32>");
    app.register_wgsl_type::<Mat4>("mat4x4<f32>");
}

pub trait RegisterWgslTypeExt {
    fn register_wgsl_type<T: 'static>(&mut self, name: &'static str) -> &mut Self;
}

impl RegisterWgslTypeExt for App {
    fn register_wgsl_type<T: 'static>(&mut self, name: &'static str) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<WgslTypes>()
            .insert(TypeId::of::<T>(), name);
        self
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct WgslTypes(TypeIdMap<&'static str>);

#[derive(Default)]
pub struct WgslType {
    pub type_name: String,
    pub snippet: Option<String>,
}

impl WgslType {
    pub fn new(type_name: impl Into<String>, snippet: Option<String>) -> Self {
        Self {
            type_name: type_name.into(),
            snippet,
        }
    }
}

impl WgslTypes {
    pub fn get_type<R: Typed>(&self) -> WgslType {
        if let Some(&name) = self.get(&TypeId::of::<R>()) {
            return WgslType::new(name, None);
        }

        let (TypeInfo::Struct(structure), Some(name)) = (R::type_info(), R::type_ident()) else {
            panic!("Render data {} is not a named struct", type_name::<R>(),)
        };

        let vars = structure.iter().fold(String::new(), |mut vars, field| {
            let wgsl_type = self.get(&field.type_id()).unwrap();
            writeln!(vars, "    {}: {},", field.name(), wgsl_type).unwrap();
            vars
        });

        let snippet = format!("struct {} {}\n{}{}\n", name, "{", vars, "}");
        WgslType::new(name, Some(snippet))
    }
}
