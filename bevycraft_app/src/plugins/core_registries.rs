use bevy::prelude::Resource;

pub struct CoreRegistries {}

#[derive(Resource, Debug, Default)]
pub enum RegistriesState {
    #[default]
    Uninit,
    Loading,
    Commited,
}
