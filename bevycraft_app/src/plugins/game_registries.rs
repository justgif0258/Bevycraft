use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevycraft_core::prelude::*;
use crate::plugins::default_registries::BLOCKS;

#[derive(Resource)]
pub struct GameRegistries {
    registries: HashMap<&'static str, RegistrySolver>
}

impl GameRegistries {
    pub const DEFAULT_NAMESPACE: &'static str = "bevycraft";
    
    pub const fn builder() -> RegistriesBuilder {
        RegistriesBuilder::new()
    }

    #[inline]
    pub fn get_registered<T: Send + Sync + 'static>(&self, key: &ResourceId) -> Option<&T> {
        self.registries
            .get(key.namespace())?
            .get_registry()?
            .get_by_path(key.path())
    }

    #[inline]
    pub fn get_registry<T: Send + Sync + 'static>(&self, namespace: &str) -> Option<&'static CompiledRegistry<T>> {
        self.registries
            .get(namespace)?
            .get_registry()
    }
}

impl Default for GameRegistries {
    fn default() -> Self {
        Self::builder()
            .add_registry(Self::DEFAULT_NAMESPACE, &BLOCKS)
            .build()
    }
}

pub struct RegistriesBuilder {
    registries: HashMap<&'static str, RegistrySolver>
}

impl RegistriesBuilder {
    pub const fn new() -> Self {
        Self { registries: HashMap::new() }
    }
    
    #[inline]
    pub fn add_registry<T: Send + Sync + 'static>(mut self, namespace: &'static str, registry: &'static CompiledRegistry<T>) -> Self {
        if let Some(solver) = self.registries.get_mut(namespace) {
            solver.add_registry(registry);
        } else {
            let mut solver = RegistrySolver::default();

            solver.add_registry(registry);

            self.registries
                .insert(namespace, solver);
        }

        self
    }
    
    #[inline]
    pub fn remove_registry<T: Send + Sync + 'static>(&mut self, namespace: &'static str) {
        if let Some(solver) = self.registries.get_mut(namespace) {
            solver.remove_registry::<T>()
        }
    }
    
    pub fn build(self) -> GameRegistries {
        GameRegistries { registries: self.registries }
    }
}