use crate::components::ConfigComponents;
use crate::configs::ConfigId;
use crate::{FinishCuttleSetup, FinishCuttleSetupSet, internal_prelude::*};
use bevy_asset::io::{AssetReaderError, MissingAssetSourceError, Reader};
use bevy_asset::{Asset, AssetApp, AssetPath, AssetServer, Handle, embedded_asset};
use code_gen::gen_shader;
use convert_case::{Case, Casing};
use derive_more::{Display, Error, From};
use std::string::FromUtf8Error;

pub mod code_gen;
pub mod wgsl_struct;

pub struct ShaderPlugin;
impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(wgsl_struct::plugin);
        app.init_asset::<Snippet>();
        app.register_type::<(Snippet, Snippets, RenderData, FunctionName, AddSnippet)>();
        app.add_systems(
            FinishCuttleSetup,
            (
                load_shaders.in_set(FinishCuttleSetupSet::LoadShaders),
                collect_component_snippets.in_set(FinishCuttleSetupSet::CollectSnippets),
            ),
        );
        embedded_asset!(app, "common.wgsl");
        embedded_asset!(app, "vertex.wgsl");
        embedded_asset!(app, "fragment.wgsl");
    }
}

#[derive(Clone)]
pub struct ComponentShaderInfo {
    pub function_name: String,
    pub data: Option<RenderData>,
}

#[derive(Clone, Component, Reflect)]
#[reflect(Component)]
pub struct RenderData {
    pub binding: u32,
    pub type_name: String,
}

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct FunctionName(pub String);

#[derive(Debug, Clone, Component)]
pub struct CuttleShader(pub Handle<Shader>);

impl FunctionName {
    pub fn from_type_name(type_name: impl Into<String>) -> Self {
        Self(type_name.into().to_case(Case::Snake))
    }
}

pub fn load_shaders(
    query: Query<(Entity, &ConfigId, &Snippets, &ConfigComponents)>,
    components: Query<(&FunctionName, Option<&RenderData>)>,
    assets: Res<AssetServer>,
    mut cmds: Commands,
) {
    for (entity, &id, snippets, comps) in &query {
        let settings = ShaderSettings {
            snippets: snippets.0.clone(),
            infos: comps
                .iter()
                .map(|&i| {
                    let (name, render_data) = components.get(i).unwrap();
                    ComponentShaderInfo {
                        function_name: name.0.clone(),
                        data: render_data.cloned(),
                    }
                })
                .collect(),
        };

        let shader = assets.add_async(load_shader(assets.clone(), settings, id.0));
        cmds.entity(entity).insert(CuttleShader(shader));
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
    println!("{}", shader);
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

pub fn collect_component_snippets(
    components: Query<&Snippets, Without<ConfigId>>,
    mut configs: Query<(&mut Snippets, &ConfigComponents), With<ConfigId>>,
) {
    for (mut config, component_entities) in &mut configs {
        for &entity in component_entities.iter() {
            config.extend_from_slice(components.get(entity).unwrap());
        }
    }
}

#[derive(Debug, Clone, Reflect)]
pub enum AddSnippet {
    Inline(String),
    File(String),
}
