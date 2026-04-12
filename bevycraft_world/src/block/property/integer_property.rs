use std::any::TypeId;
use std::str::FromStr;
use bevycraft_core::prelude::Recordable;
use crate::prelude::ErasedProperty;

pub struct IntegerProperty {
    name: &'static str,
    values: Box<[i32]>,
    min: i32,
    max: i32,
}

impl IntegerProperty {
    pub fn new(
        name: &'static str,
        min: i32,
        max: i32,
    ) -> Self {
        assert!(min >= 0, "Min value of {name} must be 0 or greater");
        assert!(max > min, "Max value of {name} must be greater than min");

        let values: Box<[i32]> = (min..=max).collect();

        Self { name, values, min, max }
    }
}

impl ErasedProperty for IntegerProperty {
    #[inline(always)]
    fn name(&self) -> &'static str {
        self.name
    }

    #[inline(always)]
    fn possible_values(&self) -> Vec<&dyn Recordable> {
        self.values
            .iter()
            .map(|i| i as &dyn Recordable)
            .collect()
    }

    #[inline(always)]
    fn get_value(&self, name: &str) -> Option<Box<dyn Recordable>> {
        i32::from_str(name)
            .ok()
            .map(|i| Box::new(i) as Box<dyn Recordable>)
    }

    #[inline(always)]
    fn get_name(&self, value: &dyn Recordable) -> Option<String> {
        value.downcast_ref::<i32>()
            .map(|i| i.to_string())
    }

    #[inline(always)]
    fn get_internal_index(&self, value: &dyn Recordable) -> Option<usize> {
        value.downcast_ref::<i32>()
            .and_then(|&i| {
                if i > self.max {
                    return None;
                }

                Some((i - self.min) as usize)
            })
    }

    #[inline(always)]
    fn type_id(&self) -> TypeId {
        TypeId::of::<i32>()
    }
}