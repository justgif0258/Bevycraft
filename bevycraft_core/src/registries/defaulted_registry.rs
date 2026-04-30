use bevy::{ecs::resource::Resource, platform::collections::HashMap};
use rapidhash::fast::RandomState;

use crate::prelude::{AssetLocation, Registrable, RegistrationError, Registry};

#[derive(Resource)]
pub struct DefaultedRegistry<T: Registrable> {
    key_to_idx: HashMap<AssetLocation, usize, RandomState>,
    idx_to_key: Vec<AssetLocation>,
    values: Vec<T>,
}

impl<T: Registrable> DefaultedRegistry<T> {
    #[inline]
    pub fn new(default_key: AssetLocation, default_value: T) -> Self {
        let mut key_to_idx = HashMap::with_hasher(RandomState::new());
        let mut idx_to_key = Vec::new();
        let mut values = Vec::new();

        key_to_idx.insert(default_key.clone(), 0);
        idx_to_key.push(default_key);
        values.push(default_value);

        Self {
            key_to_idx,
            idx_to_key,
            values,
        }
    }

    #[inline]
    pub fn get_by_key_or_default(&self, location: &AssetLocation) -> &T {
        unsafe { self.get_by_key(location).unwrap_unchecked() }
    }

    #[inline]
    pub fn get_by_idx_or_default(&self, index: usize) -> &T {
        unsafe { self.get_by_idx(index).unwrap_unchecked() }
    }

    #[inline]
    pub fn get_default(&self) -> &T {
        &self.values[0]
    }
}

impl<T: Registrable> Registry for DefaultedRegistry<T> {
    type Item = T;

    #[inline]
    fn keys(&self) -> impl Iterator<Item = &AssetLocation> {
        self.idx_to_key.iter()
    }

    #[inline]
    fn contains_key(&self, location: &AssetLocation) -> bool {
        self.key_to_idx.contains_key(location)
    }

    #[inline]
    fn get_by_key(&self, location: &AssetLocation) -> Option<&Self::Item> {
        self.key_to_idx
            .get(location)
            .and_then(|&idx| self.values.get(idx).or(Some(&self.values[0])))
    }

    #[inline]
    fn get_by_idx(&self, index: usize) -> Option<&Self::Item> {
        self.values.get(index).or(Some(&self.values[0]))
    }

    #[inline]
    fn key_to_idx(&self, location: &AssetLocation) -> Option<usize> {
        self.key_to_idx.get(location).copied()
    }

    #[inline]
    fn idx_to_key(&self, index: usize) -> Option<&AssetLocation> {
        self.idx_to_key.get(index)
    }

    #[inline]
    fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn register(
        &mut self,
        location: AssetLocation,
        value: Self::Item,
    ) -> Result<(), RegistrationError> {
        if self.contains_key(&location) {
            return Err(RegistrationError::DuplicateKey);
        }

        let idx = self.values.len();
        self.key_to_idx.insert(location.clone(), idx);
        self.idx_to_key.push(location);
        self.values.push(value);

        Ok(())
    }
}
