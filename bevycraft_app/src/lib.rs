use bevy::prelude::{Asset, Component, Handle, Resource, States, UntypedHandle};
use bevy::utils::TypeIdMap;
use std::any::TypeId;

pub mod systems;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    ModelDiscovery,
    AwaitModels,
    BuildArrayTexture,
    CacheMeshes,
    Finishing,
    InGame,
}

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
pub struct AssetsLoading(TypeIdMap<Vec<UntypedHandle>>);

impl AssetsLoading {
    pub fn add<T: Asset>(&mut self, handle: Handle<T>) {
        let type_id = TypeId::of::<T>();

        let assets = if let Some(vec) = self.0.get_mut(&type_id) {
            vec
        } else {
            self.0.insert(type_id, Vec::new());

            self.0.get_mut(&type_id).unwrap()
        };

        assets.push(handle.untyped());
    }

    pub fn get<T: Asset>(&self, index: usize) -> Option<Handle<T>> {
        self.0.get(&TypeId::of::<T>()).and_then(|vec| {
            vec.get(index)
                .map(|untyped| untyped.clone().typed_unchecked::<T>())
        })
    }

    pub fn iter<T: Asset>(&self) -> impl Iterator<Item = Handle<T>> {
        let type_id = TypeId::of::<T>();

        self.0
            .get(&type_id)
            .map(|vec| {
                vec.iter()
                    .cloned()
                    .map(|untyped| untyped.typed_unchecked::<T>())
            })
            .unwrap()
    }
}
