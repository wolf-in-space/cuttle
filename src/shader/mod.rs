use crate::{calculations::Calculations, components::SdfCompInfos};
use bevy::{
    asset::{embedded_asset, io::Reader, AssetLoader, LoadContext, LoadDirectError},
    prelude::*,
};
use derive_more::derive::{Display, Error, From};
use gen::gen_shader;
use snippets::{AddSnippet, AddSnippets, Snippet, SnippetPlugin};
use wgsl_struct::WgslTypeInfos;

pub mod gen;
pub mod snippets;
pub mod wgsl_struct;

pub struct ShaderPlugin;
impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((wgsl_struct::plugin, SnippetPlugin));

        embedded_asset!(app, "dummy.dummy");

        embedded_asset!(app, "common.wgsl");
        embedded_asset!(app, "vertex.wgsl");
        embedded_asset!(app, "fragment.wgsl");
    }

    fn finish(&self, app: &mut App) {
        let world = app.world_mut();

        let infos = world.remove_resource::<SdfCompInfos>().unwrap();
        let wgsl_types = world.remove_resource::<WgslTypeInfos>().unwrap();
        let calcs = world.remove_resource::<Calculations>().unwrap();
        let snippets = world.remove_resource::<AddSnippets>().unwrap();

        let loader = ShaderLoader {
            infos,
            wgsl_types,
            calcs,
            snippets,
        };

        let assets = world.resource_mut::<AssetServer>();
        assets.register_loader(loader);
        let shader = assets.load("embedded://bevy_comdf/shader/dummy.dummy");

        world.insert_resource(GeneratedShader { shader });
    }
}

#[derive(Asset, Reflect)]
pub struct SdfShaderImport(String);

#[derive(Resource)]
pub struct GeneratedShader {
    pub shader: Handle<Shader>,
}

struct ShaderLoader {
    infos: SdfCompInfos,
    wgsl_types: WgslTypeInfos,
    calcs: Calculations,
    snippets: AddSnippets,
}

#[derive(Debug, Error, Display, From)]
enum ShaderLoaderError {
    Load(LoadDirectError),
}

impl AssetLoader for ShaderLoader {
    type Asset = Shader;
    type Settings = ();
    type Error = ShaderLoaderError;

    async fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut snippets = String::new();
        for add in self.snippets.iter() {
            let Snippet(snippet) = match add {
                AddSnippet::Inline(snippet) => Snippet(snippet.clone()),
                AddSnippet::File(path) => {
                    load_context.loader().immediate().load(path).await?.take()
                }
            };
            snippets.push_str(&snippet);
        }

        let shader = gen_shader(&self.infos, &self.wgsl_types, &self.calcs, snippets);
        let shader = Shader::from_wgsl(shader, format!("Generated at {} | {}", file!(), line!()));

        Ok(shader)
    }

    fn extensions(&self) -> &[&str] {
        &["dummy"]
    }
}
