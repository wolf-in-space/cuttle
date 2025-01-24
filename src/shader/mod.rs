use crate::calculations::Calculation;
use crate::groups::global::GlobalGroupInfos;
use crate::pipeline::specialization::CuttlePipeline;
use crate::shader::wgsl_struct::ToWgslFn;
use bevy::asset::io::{AssetReaderError, MissingAssetSourceError};
use bevy::asset::AssetPath;
use bevy::render::RenderApp;
use bevy::{
    asset::{embedded_asset, io::Reader},
    prelude::*,
};
use derive_more::derive::{Display, Error, From};
use gen::gen_shader;
use std::string::FromUtf8Error;

pub mod gen;
pub mod wgsl_struct;

pub struct ShaderPlugin;
impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(wgsl_struct::plugin);
        app.init_asset::<Snippet>().init_resource::<AddSnippets>();

        embedded_asset!(app, "common.wgsl");
        embedded_asset!(app, "vertex.wgsl");
        embedded_asset!(app, "fragment.wgsl");
    }
}

pub struct ComponentShaderInfo {
    pub function_name: String,
    pub render_data: Option<RenderDataShaderInfo>,
}

pub struct ToComponentShaderInfo {
    pub function_name: String,
    pub to_render_data: Option<ToRenderDataShaderInfo>,
}

#[derive(Clone)]
pub struct RenderDataShaderInfo {
    pub binding: u32,
    pub wgsl: RenderDataWgsl,
}

pub struct ToRenderDataShaderInfo {
    pub binding: u32,
    pub to_wgsl: ToWgslFn,
}

#[derive(Clone)]
pub struct RenderDataWgsl {
    pub definition: String,
    pub name: String,
}

pub(crate) fn load_shader_to_pipeline(app: &mut App, settings: ShaderSettings, group_id: usize) {
    let comp_count = app
        .world()
        .resource::<GlobalGroupInfos>()
        .component_bindings
        .len() as u32;

    let assets = app.world().resource::<AssetServer>();
    let shader = assets.add_async(load_shader(assets.clone(), settings, group_id));

    let render_world = app.sub_app_mut(RenderApp).world_mut();
    match render_world.get_resource_mut::<CuttlePipeline>() {
        Some(mut pipeline) => {
            pipeline.fragment_shaders.insert(group_id, shader);
        }
        None => {
            let mut pipeline = CuttlePipeline::new(render_world, comp_count);
            pipeline.fragment_shaders.insert(group_id, shader);
            render_world.insert_resource(pipeline);
        }
    }
}

#[derive(Debug, Error, Display, From)]
enum LoadShaderError {
    AssetSource(MissingAssetSourceError),
    Read(AssetReaderError),
    IO(std::io::Error),
    Utf8(FromUtf8Error),
}

async fn load_shader(
    assets: AssetServer,
    settings: ShaderSettings,
    group_id: usize,
) -> Result<Shader, LoadShaderError> {
    let mut snippets = String::new();
    let base = [AddSnippet::File(
        "embedded://cuttle/shader/fragment.wgsl".to_string(),
    )];
    let snippet_sources = base.into_iter().chain(settings.snippets);
    for add in snippet_sources {
        let Snippet(snippet) = match add {
            AddSnippet::Inline(snippet) => Snippet(snippet.clone()),
            AddSnippet::File(path) => {
                let bytes = load_asset_bytes_manually(&assets, path).await?;
                Snippet(String::from_utf8(bytes)?)
            }
        };
        snippets.push_str(&snippet);
    }

    let shader = gen_shader(&settings.infos, &settings.calculations, snippets);
    // println!("{}", shader);
    let shader = Shader::from_wgsl(
        shader,
        format!("Generated at {} | {}: {:?}", file!(), line!(), group_id),
    );
    Ok(shader)
}

async fn load_asset_bytes_manually(
    assets: &AssetServer,
    path: String,
) -> Result<Vec<u8>, LoadShaderError> {
    let path = AssetPath::from(path);
    let mut reader = assets
        .get_source(path.source())?
        .reader()
        .read(path.path())
        .await?;
    let mut bytes = Vec::new();
    Reader::read_to_end(&mut reader, &mut bytes).await?;
    Ok(bytes)
}

#[derive(Default)]
pub(crate) struct ShaderSettings {
    pub infos: Vec<ComponentShaderInfo>,
    pub calculations: Vec<Calculation>,
    pub snippets: Vec<AddSnippet>,
}

#[derive(Asset, TypePath, Debug)]
pub struct Snippet(pub String);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AddSnippets(Vec<AddSnippet>);

#[derive(Clone)]
pub enum AddSnippet {
    Inline(String),
    File(String),
}
