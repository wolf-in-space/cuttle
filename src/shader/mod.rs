use crate::calculations::Calculation;
use crate::components::initialization::ComponentShaderInfo;
use crate::groups::{GlobalGroupInfos, GroupId};
use crate::pipeline::specialization::SdfPipeline;
use bevy::render::RenderApp;
use bevy::{
    asset::{embedded_asset, io::Reader, AssetLoader, LoadContext, LoadDirectError},
    prelude::*,
};
use derive_more::derive::{Display, Error, From};
use gen::gen_shader;
use serde::{Deserialize, Serialize};
use snippets::{AddSnippet, Snippet, SnippetPlugin};

pub mod gen;
pub mod snippets;
pub mod wgsl_struct;

pub struct ShaderPlugin;
impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((wgsl_struct::plugin, SnippetPlugin));
        app.init_asset_loader::<ShaderLoader>();

        embedded_asset!(app, "dummy.dummy");

        embedded_asset!(app, "common.wgsl");
        embedded_asset!(app, "vertex.wgsl");
        embedded_asset!(app, "fragment.wgsl");
    }
}

pub(crate) fn load_shader_to_pipeline(
    app: &mut App,
    shader_settings: ShaderSettings,
    group_id: GroupId,
) {
    let comp_count = app
        .world()
        .resource::<GlobalGroupInfos>()
        .component_bindings
        .len() as u32;
    let assets = app.world_mut().resource_mut::<AssetServer>();
    let shader = assets.load_with_settings("embedded://bevy_comdf/shader/dummy.dummy", move |s| {
        *s = shader_settings.clone()
    });
    let render_world = app.sub_app_mut(RenderApp).world_mut();
    match render_world.get_resource_mut::<SdfPipeline>() {
        Some(mut pipeline) => {
            pipeline.fragment_shaders.insert(group_id, shader);
        }
        None => {
            let mut pipeline = SdfPipeline::new(render_world, comp_count);
            pipeline.fragment_shaders.insert(group_id, shader);
            render_world.insert_resource(pipeline);
        }
    }
}

#[derive(Asset, Reflect)]
pub struct SdfShaderImport(String);

#[derive(Default)]
struct ShaderLoader;

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct ShaderSettings {
    pub infos: Vec<ComponentShaderInfo>,
    pub calculations: Vec<Calculation>,
    pub snippets: Vec<AddSnippet>,
}

#[derive(Debug, Error, Display, From)]
enum ShaderLoaderError {
    Load(LoadDirectError),
}

impl AssetLoader for ShaderLoader {
    type Asset = Shader;
    type Settings = ShaderSettings;
    type Error = ShaderLoaderError;

    async fn load(
        &self,
        _reader: &mut dyn Reader,
        settings: &ShaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut snippets = String::new();
        let base = [AddSnippet::File(
            "embedded://bevy_comdf/shader/fragment.wgsl".to_string(),
        )];
        let snippet_sources = base.iter().chain(&settings.snippets);
        for add in snippet_sources {
            let Snippet(snippet) = match add {
                AddSnippet::Inline(snippet) => Snippet(snippet.clone()),
                AddSnippet::File(path) => {
                    load_context.loader().immediate().load(path).await?.take()
                }
            };
            snippets.push_str(&snippet);
        }

        let shader = gen_shader(&settings.infos, &settings.calculations, snippets);
        let shader = Shader::from_wgsl(shader, format!("Generated at {} | {}", file!(), line!()));

        Ok(shader)
    }

    fn extensions(&self) -> &[&str] {
        &["dummy"]
    }
}
