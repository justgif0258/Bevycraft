use std::any::TypeId;
use bevycraft_core::prelude::Recordable;
use crate::prelude::ErasedProperty;

pub struct BooleanProperty {
    name: &'static str,
}

impl BooleanProperty {
    const VALUES: [bool; 2] = [true, false];

    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl ErasedProperty for BooleanProperty {
    #[inline(always)]
    fn name(&self) -> &'static str {
        self.name
    }

    #[inline(always)]
    fn possible_values(&self) -> Vec<&dyn Recordable> {
        Self::VALUES
            .iter()
            .map(|value| value as &dyn Recordable)
            .collect()
    }

    #[inline(always)]
    fn get_value(&self, name: &str) -> Option<Box<dyn Recordable>> {
        match name {
            "true" => Some(Box::new(true)),
            "false" => Some(Box::new(false)),
            _ => None,
        }
    }

    #[inline(always)]
    fn get_name(&self, value: &dyn Recordable) -> Option<String> {
        value.downcast_ref::<bool>()
            .map(|b| b.to_string())
    }

    #[inline(always)]
    fn get_internal_index(&self, value: &dyn Recordable) -> Option<usize> {
        value.downcast_ref::<bool>()
            .map(|&b| b as usize)
    }

    #[inline(always)]
    fn type_id(&self) -> TypeId {
        TypeId::of::<bool>()
    }
}