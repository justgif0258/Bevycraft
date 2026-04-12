use bevy::platform::hash::NoOpHash;
use frozen_collections::FzHashMap;
use bevycraft_core::prelude::Recordable;
use crate::prelude::{Definition, ErasedDefinition};

#[derive(Debug)]
pub struct BlockDefinition {
    definitions: FzHashMap<&'static dyn ErasedDefinition, Box<dyn Recordable>, NoOpHash>,
}

impl BlockDefinition {
    pub const fn new() -> BlockDefinitionBuilder {
        BlockDefinitionBuilder(Vec::new())
    }

    #[inline(always)]
    pub fn get<T: Recordable>(&self, definition: &'static Definition<T>) -> &T {
        if let Some(value) = self.definitions.get(definition.as_erased()) {
            unsafe {
                return value.as_ref()
                    .downcast_ref_unchecked()
            }
        }

        definition.default()
    }
}

pub struct BlockDefinitionBuilder(Vec<(&'static dyn ErasedDefinition, Box<dyn Recordable>)>);

impl BlockDefinitionBuilder {
    #[inline(always)]
    pub fn add<T: Recordable>(mut self, definition: &'static Definition<T>, value: T) -> Self {
        assert!(!self.contains(definition), "Tried adding duplicate '{}' definition", definition.name());

        self.0.push((
            definition,
            Box::new(definition.normalize(value)),
        ));

        self
    }

    #[inline(always)]
    pub fn build(self) -> BlockDefinition {
        BlockDefinition {
            definitions: FzHashMap::with_hasher(self.0, NoOpHash)
        }
    }

    #[inline(always)]
    fn contains<T: Recordable>(&self, definition: &'static Definition<T>) -> bool {
        self.0.iter().find(|(s, _)| *s == definition.as_erased()).is_some()
    }
}