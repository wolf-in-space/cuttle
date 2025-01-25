use crate::calculations::{Calculation, Calculations};
use crate::components::ComponentInfos;
use crate::groups::GroupId;
use crate::shader::wgsl_struct::{ToWgslFn, WgslTypeInfos};
use bevy::asset::io::{AssetReaderError, MissingAssetSourceError};
use bevy::asset::AssetPath;
use bevy::utils::HashMap;
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
        app.init_asset::<Snippet>();

        embedded_asset!(app, "common.wgsl");
        embedded_asset!(app, "vertex.wgsl");
        embedded_asset!(app, "fragment.wgsl");
    }
}

pub struct ComponentShaderInfo {
    pub function_name: String,
    pub render_data: Option<ShaderInfo>,
}

#[derive(Debug, Reflect)]
pub struct ToComponentShaderInfo {
    pub function_name: String,
    #[reflect(ignore)]
    pub to_render_data: Option<ToRenderData>,
}

#[derive(Clone)]
pub struct ShaderInfo {
    pub binding: u32,
    pub wgsl: RenderDataWgsl,
}

#[derive(Debug, Copy, Clone)]
pub struct ToRenderData {
    pub binding: u32,
    pub to_wgsl: ToWgslFn,
}

#[derive(Clone)]
pub struct RenderDataWgsl {
    pub definition: String,
    pub name: String,
}

pub fn load_shaders(
    query: Query<(&GroupId, &ComponentInfos, &Snippets, &Calculations)>,
    wgsl_type_infos: Res<WgslTypeInfos>,
    assets: Res<AssetServer>,
) -> HashMap<GroupId, Handle<Shader>> {
    let mut result = HashMap::new();
    let wgsl_type_infos = wgsl_type_infos.into_inner();

    for (&id, infos, snippets, calculations) in &query {
        let settings = ShaderSettings {
            snippets: snippets.0.clone(),
            calculations: calculations.0.clone(),
            infos: infos
                .iter()
                .map(|i| ComponentShaderInfo {
                    function_name: i.to_shader_info.function_name.clone(),
                    render_data: i.to_shader_info.to_render_data.clone().map(
                        |ToRenderData { binding, to_wgsl }| ShaderInfo {
                            binding,
                            wgsl: to_wgsl(wgsl_type_infos),
                        },
                    ),
                })
                .collect(),
        };

        let shader = assets.add_async(load_shader(assets.clone(), settings, id.0));
        result.insert(id, shader);
    }

    result
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

#[derive(Asset, Debug, Reflect)]
pub struct Snippet(pub String);

#[derive(Debug, Component, Default, Deref, DerefMut, Reflect)]
pub struct Snippets(Vec<AddSnippet>);

#[derive(Debug, Clone, Reflect)]
pub enum AddSnippet {
    Inline(String),
    File(String),
}
