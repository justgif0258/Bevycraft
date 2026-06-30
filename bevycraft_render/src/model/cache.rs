use {
    crate::prelude::Model,
    bevy::prelude::Resource,
    bevycraft_core::prelude::Registrable,
    std::{marker::PhantomData, sync::Arc},
};

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
        self.cache.get(index)
            .map(|m| m.as_ref())
            .flatten()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = Option<&M>> {
        self.cache.iter().map(Option::as_ref)
    }
}
