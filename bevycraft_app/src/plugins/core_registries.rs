use bevy::prelude::Resource;
use bevycraft_core::prelude::Record;
use bevycraft_world::prelude::Block;

pub struct CoreRegistries {
    pub blocks: Record<Block>,
}

#[derive(Resource, Debug, Default)]
pub enum RegistriesState {
    #[default]
    Uninit,
    Loading,
    Commited,
}