use bevy::asset::io::Reader;
use bevy::asset::{
    Asset,
    AssetLoader,
    LoadContext,
};
use bevy::prelude::TypePath;
use serde::Deserialize;
use thiserror::Error;

#[derive(TypePath, Default)]
pub(crate) struct RonAssetLoader<T>
where
    T: Asset + TypePath + for<'de> Deserialize<'de>,
{
    _unused: Option<T>,
}

#[derive(Debug, Error)]
pub(crate) enum RonAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl<T> AssetLoader for RonAssetLoader<T>
where
    T: Asset + TypePath + for<'de> Deserialize<'de>,
{
    type Asset = T;
    type Settings = ();
    type Error = RonAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<T>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}
