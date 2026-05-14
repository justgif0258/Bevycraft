pub mod block_model;
pub mod manager;
pub mod r_model;
pub mod cache;

pub trait Model: bevy::asset::Asset + bevy::reflect::TypePath + Sized {
    type Error: std::error::Error + Send + Sync + 'static;

    fn resolve(
        raw: crate::prelude::RModel,
        textures: &crate::prelude::TextureManager<Self>,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = Result<Self, Self::Error>>;
}

#[derive(thiserror::Error, Debug)]
pub enum ModelLoadError<E: std::error::Error + Send + Sync + 'static> {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("byte reading error: {0}")]
    ReadBytesError(#[from] bevy::asset::ReadAssetBytesError),

    #[error("failed to deserialize RON model: {0}")]
    Deserialize(#[from] ron::de::SpannedError),

    #[error("invalid location: {0}")]
    InvalidLocation(#[from] bevycraft_core::prelude::AssetLocationError),

    #[error("undefined texture: {0}")]
    UndefinedTexture(Box<str>),

    #[error(transparent)]
    Resolve(E),
}
