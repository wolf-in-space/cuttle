use crate::internal_prelude::*;
use crate::shader::RenderDataWgsl;
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
            .get_resource_or_init::<WgslTypeInfos>()
            .insert(TypeId::of::<T>(), name);
        self
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct WgslTypeInfos(TypeIdMap<&'static str>);

pub type ToWgslFn = fn(&WgslTypeInfos) -> RenderDataWgsl;

impl WgslTypeInfos {
    pub fn wgsl_type_for_struct<R: Typed>(&self) -> RenderDataWgsl {
        let (TypeInfo::Struct(structure), Some(name)) = (R::type_info(), R::type_ident()) else {
            panic!("Render data {} is not a named struct", type_name::<R>(),)
        };

        let vars = structure.iter().fold(String::new(), |mut vars, field| {
            let wgsl_type = self.get(&field.type_id()).unwrap();
            writeln!(vars, "    {}: {},", field.name(), wgsl_type).unwrap();
            vars
        });
        let definition = format!("struct {} {}\n{}{}\n", name, "{", vars, "}");

        RenderDataWgsl {
            definition,
            name: name.to_string(),
        }
    }

    pub fn wgsl_type_for_builtin<R: Typed>(&self) -> RenderDataWgsl {
        let Some(wgsl_type) = self.get(&TypeId::of::<R>()) else {
            panic!(
                "RenderData {} not registered with WgslTypeInfos",
                type_name::<R>(),
            )
        };

        RenderDataWgsl {
            name: wgsl_type.to_string(),
            definition: String::new(),
        }
    }
}
