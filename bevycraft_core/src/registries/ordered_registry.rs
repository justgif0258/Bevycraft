use {
    crate::{
        prelude::{AssetLocation, Registrable, RegistrationError},
        registries::registry::Registry,
    },
    bevy::{ecs::resource::Resource, platform::collections::HashMap},
    rapidhash::fast::RandomState,
};

#[derive(Resource)]
pub struct OrderedRegistry<T: Registrable> {
    key_to_idx: HashMap<AssetLocation, usize, RandomState>,
    idx_to_key: Vec<AssetLocation>,
    values: Vec<T>,

    frozen: bool,
}

impl<T: Registrable> OrderedRegistry<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            key_to_idx: HashMap::with_hasher(RandomState::new()),
            idx_to_key: Vec::new(),
            values: Vec::new(),
            frozen: false,
        }
    }
}

impl<T: Registrable> Registry<T> for OrderedRegistry<T> {
    #[inline]
    fn iter(&self) -> impl Iterator<Item = (&AssetLocation, &T)> {
        self.key_to_idx
            .iter()
            .map(|(key, &idx)| (key, &self.values[idx]))
    }

    #[inline]
    fn keys(&self) -> impl Iterator<Item = &AssetLocation> {
        self.key_to_idx.keys()
    }

    #[inline]
    fn contains_key(&self, location: &AssetLocation) -> bool {
        self.key_to_idx.contains_key(location)
    }

    #[inline]
    fn get_by_key(&self, location: &AssetLocation) -> Option<&T> {
        self.key_to_idx
            .get(location)
            .copied()
            .map(|idx| &self.values[idx])
    }

    #[inline]
    fn get_by_idx(&self, index: usize) -> Option<&T> {
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
    fn register(&mut self, location: AssetLocation, value: T) -> Result<&T, RegistrationError> {
        if self.frozen {
            return Err(RegistrationError::FrozenRegistry);
        }

        if self.key_to_idx.contains_key(&location) {
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
