use bevy::{asset::embedded_asset, prelude::*};
use gen::gen_shader;
use wgsl_struct::WgslTypeInfos;

use crate::{calculations::Calculations, components::SdfCompInfos};

pub mod gen;
pub mod wgsl_struct;

pub struct ShaderPlugin;
impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(wgsl_struct::plugin);
        app.init_resource::<ShaderImports>();

        embedded_asset!(app, "common.wgsl");
        embedded_asset!(app, "vertex.wgsl");
        embedded_asset!(app, "fragment.wgsl");
    }

    fn finish(&self, app: &mut App) {
        let world = app.world_mut();
        let infos = world.resource::<SdfCompInfos>();

        let wgsl_types = world.resource::<WgslTypeInfos>();
        let shader_imports = world.resource::<ShaderImports>();
        let calculations = world.resource::<Calculations>();

        let definitions = gen_shader(infos, wgsl_types, calculations, shader_imports);
        let definitions = Shader::from_wgsl(
            definitions,
            format!("Generated at {} | {}", file!(), line!()),
        );

        let mut shaders = world.resource_mut::<Assets<Shader>>();
        let definitions = shaders.add(definitions);

        world.insert_resource(GeneratedShaders { definitions });
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ShaderImports(Vec<String>);

#[derive(Resource)]
pub struct GeneratedShaders {
    pub definitions: Handle<Shader>,
}
