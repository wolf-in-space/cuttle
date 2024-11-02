use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};
use derive_more::derive::{Display, Error, From};
use serde::Deserialize;

pub struct SnippetPlugin;
impl Plugin for SnippetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Snippet>()
            .init_asset_loader::<ShaderSnippetsLoader>()
            .init_resource::<AddSnippets>();
    }

    fn finish(&self, _app: &mut App) {}
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct Snippet(pub String);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AddSnippets(Vec<AddSnippet>);

pub enum AddSnippet {
    Inline(String),
    File(String),
}

#[derive(Default)]
struct ShaderSnippetsLoader;

#[derive(Debug, Error, Display, From)]
enum ShaderSnippetsLoaderError {
    Io(std::io::Error),
    String(std::string::FromUtf8Error),
}

impl AssetLoader for ShaderSnippetsLoader {
    type Asset = Snippet;
    type Settings = ();
    type Error = ShaderSnippetsLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let snippet = String::from_utf8(bytes)?;
        Ok(Snippet(snippet))
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }
}
