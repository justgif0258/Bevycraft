use bevy::asset::AssetServer;
use bevy::ecs::resource::Resource;
use bevy::prelude::{Assets, Handle};
use bevycraft_core::prelude::Registrable;
use std::marker::PhantomData;
use std::sync::Arc;
use crate::prelude::{Model, ModelCache};

#[derive(Resource, Default)]
pub struct ModelManager<T: Registrable, M: Model> {
    handles: Vec<Option<Handle<M>>>,
    _marker: PhantomData<fn(T)>,
}

impl<T: Registrable, M: Model> ModelManager<T, M> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            handles: vec![None; capacity],
            _marker: PhantomData,
        }
    }

    pub fn build_cache(self, models: &mut Assets<M>) -> ModelCache<T, M> {
        let boxed = self.handles
            .into_iter()
            .map(|h| {
                if let Some(h) = h {
                    return models.remove(&h)
                }

                None
            });

        ModelCache {
            cache: Arc::<[Option<M>]>::from_iter(boxed),
            _marker: PhantomData,
        }
    }

    pub fn set(&mut self, index: usize, handle: Handle<M>) {
        if index >= self.handles.len() {
            self.handles.resize(index + 1, None);
        }

        self.handles[index] = Some(handle);
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&Handle<M>> {
        self.handles.get(index)?.as_ref()
    }

    #[inline]
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn get_unchecked(&self, index: usize) -> &Handle<M> {
        self.handles
            .get_unchecked(index)
            .as_ref()
            .unwrap_unchecked()
    }

    pub fn is_fully_populated(&self) -> bool {
        self.handles.iter().all(|h| h.is_some())
    }

    pub fn is_all_loaded(&self, server: &AssetServer) -> bool {
        self.handles
            .iter()
            .filter_map(|h| h.as_ref())
            .all(|h| server.is_loaded_with_dependencies(h))
    }
}
