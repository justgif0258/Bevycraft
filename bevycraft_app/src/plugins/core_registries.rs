use bevy::prelude::Resource;
use bevycraft_core::prelude::MappedRecord;
use bevycraft_world::prelude::Block;

pub struct CoreRegistries {
    pub blocks: MappedRecord<Box<dyn Block>>,
}

#[derive(Resource, Debug, Default)]
pub enum RegistriesState {
    #[default]
    Uninit,
    Loading,
    Commited,
}
