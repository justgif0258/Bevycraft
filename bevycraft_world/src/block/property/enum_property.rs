use std::any::TypeId;
use frozen_collections::FzStringMap;
use bevycraft_core::prelude::Recordable;
use crate::prelude::ErasedProperty;

pub struct EnumProperty<E: Recordable + ToString + Copy + PartialEq> {
    name        : &'static str,
    values      : &'static [E],
    names       : FzStringMap<Box<str>, E>,
}

impl<E: Recordable + ToString + Copy + PartialEq> EnumProperty<E> {
    pub fn new(
        name: &'static str,
        values: &'static [E],
    ) -> Self {
        let names: FzStringMap<Box<str>, E> = values
            .iter()
            .map(|e| {
                (e.to_string().into_boxed_str(), *e)
            })
            .collect();

        Self {
            name,
            values,
            names,
        }
    }
}

impl<E: Recordable + ToString + Copy + PartialEq> ErasedProperty for EnumProperty<E> {
    #[inline(always)]
    fn name(&self) -> &'static str {
        self.name
    }

    #[inline(always)]
    fn possible_values(&self) -> Vec<&dyn Recordable> {
        self.values
            .iter()
            .map(|e| e as &dyn Recordable)
            .collect()
    }

    #[inline(always)]
    fn get_value(&self, name: &str) -> Option<Box<dyn Recordable>> {
        self.names.get(name).map(|&e| Box::new(e) as Box<dyn Recordable>)
    }

    #[inline(always)]
    fn get_name(&self, value: &dyn Recordable) -> Option<String> {
        value.downcast_ref::<E>()
            .map(|e| e.to_string())
    }

    #[inline(always)]
    fn get_internal_index(&self, value: &dyn Recordable) -> Option<usize> {
        let downcast = value.downcast_ref::<E>()?;

        self.values
            .iter()
            .position(|e| e == downcast)
    }

    #[inline(always)]
    fn type_id(&self) -> TypeId {
        TypeId::of::<E>()
    }
}