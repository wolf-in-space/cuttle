use crate::utils::GetOrInitResourceWorldExt;
use bevy::reflect::StructInfo;
use bevy::{prelude::*, utils::TypeIdMap};
use std::{any::TypeId, fmt::Write};

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
        let mut reg = self.world_mut().resource_or_init::<WgslTypeInfos>();
        reg.register::<T>(name);
        self
    }
}

#[derive(Debug, Clone)]
pub struct WgslTypeInfo {
    pub name: &'static str,
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct WgslTypeInfos(TypeIdMap<WgslTypeInfo>);

impl WgslTypeInfos {
    pub fn structure_to_wgsl(&self, structure: &StructInfo, name: &str) -> String {
        let vars: String = structure.iter().fold(String::new(), |mut accu, field| {
            let wgsl_type_name = self.get(&field.type_id()).unwrap().name;
            writeln!(accu, "    {}: {},", field.name(), wgsl_type_name).unwrap();
            accu
        });
        format!("struct {} {}\n{}{}\n", name, "{", vars, "}")
    }

    pub fn register<T: 'static>(&mut self, name: &'static str) {
        self.insert(TypeId::of::<T>(), WgslTypeInfo { name });
    }
}
