use bevy::{ecs::resource::Resource, platform::collections::HashMap};
use rapidhash::fast::RandomState;

use crate::prelude::{AssetLocation, Registrable, RegistrationError, Registry};

#[derive(Resource)]
pub struct DefaultedRegistry<T: Registrable> {
    key_to_idx: HashMap<AssetLocation, usize, RandomState>,
    idx_to_key: Vec<AssetLocation>,
    values: Vec<T>,
    frozen: bool,
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
            frozen: false,
        }
    }

    #[inline]
    pub fn get_by_key_or_default(&self, location: &AssetLocation) -> &T {
        self.get_by_key(location).unwrap_or(&self.default_value())
    }

    #[inline]
    pub fn get_by_idx_or_default(&self, index: usize) -> &T {
        self.get_by_idx(index).unwrap_or(&self.default_value())
    }

    #[inline]
    pub fn key_to_idx_or_default(&self, location: &AssetLocation) -> usize {
        self.key_to_idx.get(location).copied().unwrap_or(0)
    }

    #[inline]
    pub fn idx_to_key_or_default(&self, index: usize) -> &AssetLocation {
        self.idx_to_key.get(index).unwrap_or(&self.default_key())
    }

    #[inline]
    pub fn default_key(&self) -> &AssetLocation {
        &self.idx_to_key[0]
    }

    #[inline]
    pub fn default_value(&self) -> &T {
        &self.values[0]
    }
}

impl<T: Registrable> Registry for DefaultedRegistry<T> {
    type Item = T;

    #[inline]
    fn iter(&self) -> impl Iterator<Item = (&AssetLocation, &Self::Item)> {
        self.key_to_idx
            .iter()
            .map(|(key, &idx)| (key, &self.values[idx]))
    }

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
            .copied()
            .map(|idx| &self.values[idx])
    }

    #[inline]
    fn get_by_idx(&self, index: usize) -> Option<&Self::Item> {
        self.values.get(index)
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
    fn frozen(&self) -> bool {
        self.frozen
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
    ) -> Result<&Self::Item, RegistrationError> {
        if self.frozen {
            return Err(RegistrationError::FrozenRegistry);
        }

        if self.contains_key(&location) {
            if &location == self.default_key() {
                self.values[0] = value;

                return Ok(self.default_value());
            }

            return Err(RegistrationError::DuplicateKey);
        }

        let idx = self.values.len();
        self.key_to_idx.insert(location.clone(), idx);
        self.idx_to_key.push(location);
        self.values.push(value);

        Ok(self.values.last().unwrap())
    }

    #[inline]
    fn freeze(&mut self) {
        self.frozen = true;
    }
}
