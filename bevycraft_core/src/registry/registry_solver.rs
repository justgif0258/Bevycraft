use std::any::{Any, TypeId};
use bevy::utils::TypeIdMap;
use crate::prelude::CompiledRegistry;

pub struct RegistrySolver {
    solver: TypeIdMap<&'static (dyn Any + Send + Sync)>,
}

impl Default for RegistrySolver {
    fn default() -> RegistrySolver {
        Self { solver: TypeIdMap::default() }
    }
}

impl RegistrySolver {
    #[inline]
    pub fn add_registry<T: Send + Sync + 'static>(&mut self, registry: &'static CompiledRegistry<T>) {
        debug_assert!(!self.solver.contains_key(&TypeId::of::<T>()), "Cannot add duplicate registry type to solver");

        self.solver
            .insert(
                TypeId::of::<T>(),
                registry
            );
    }

    #[inline]
    pub fn remove_registry<T: Send + Sync + 'static>(&mut self) {
        self.solver.remove(&TypeId::of::<T>());
    }

    #[inline]
    pub fn get_registry<T: Send + Sync + 'static>(&self) -> Option<&'static CompiledRegistry<T>> {
        self.solver
            .get(&TypeId::of::<T>())?
            .downcast_ref::<CompiledRegistry<T>>()
    }
}