use crate::components::ConfigComponents;
use crate::configs::ConfigId;
use crate::internal_prelude::*;
use bevy_asset::io::{AssetReaderError, MissingAssetSourceError, Reader};
use bevy_asset::{embedded_asset, Asset, AssetApp, AssetPath, AssetServer, Handle};
use derive_more::{Display, Error, From};
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

#[derive(Clone)]
pub struct ComponentShaderInfo {
    pub name: ComponentName,
    pub binding: Option<u32>,
}

#[derive(Clone, Component)]
pub struct Binding(u32);

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct ComponentName {
    pub type_name: String,
    pub function_name: String,
}

pub fn load_shaders(
    query: Query<(&ConfigId, &Snippets, &ConfigComponents)>,
    components: Query<(&ComponentName, Option<&Binding>)>,
    assets: Res<AssetServer>,
) -> HashMap<ConfigId, Handle<Shader>> {
    let mut result = HashMap::new();

    for (&id, snippets, comps) in &query {
        let settings = ShaderSettings {
            snippets: snippets.0.clone(),
            infos: comps
                .iter()
                .map(|&i| {
                    let (name, binding) = components.get(i).unwrap();
                    ComponentShaderInfo {
                        name: name.clone(),
                        binding: binding.map(|b| b.0).clone(),
                    }
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
        "embedded://cuttle_core/shader/fragment.wgsl".to_string(),
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

    let shader = gen_shader(&settings.infos, snippets);
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
    pub snippets: Vec<AddSnippet>,
}

#[derive(Asset, Debug, Reflect, Component)]
pub struct Snippet(pub String);

#[derive(Debug, Component, Default, Deref, DerefMut, Reflect)]
pub struct Snippets(Vec<AddSnippet>);

#[derive(Debug, Clone, Reflect)]
pub enum AddSnippet {
    Inline(String),
    File(String),
}
