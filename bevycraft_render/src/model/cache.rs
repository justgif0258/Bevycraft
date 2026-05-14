use crate::prelude::{Model, ModelManager};
use bevy::prelude::{AssetServer, Resource};
use bevycraft_core::prelude::Registrable;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Resource)]
pub struct ModelCache<T: Registrable, M: Model> {
    pub(crate) cache: Arc<[Option<M>]>,
    pub(crate) _marker: PhantomData<fn(T)>,
}

impl<T: Registrable, M: Model> Clone for ModelCache<T, M> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Registrable, M: Model> ModelCache<T, M> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<&M> {
        self.cache[index].as_ref()
    }
}
