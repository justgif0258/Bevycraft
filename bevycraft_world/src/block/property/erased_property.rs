use std::any::TypeId;
use std::hash::{Hash, Hasher};
use bevycraft_core::prelude::Recordable;

pub trait ErasedProperty: Recordable {
    fn name(&self) -> &'static str;

    fn possible_values(&self) -> Vec<&dyn Recordable>;

    fn get_value(&self, name: &str) -> Option<Box<dyn Recordable>>;

    fn get_name(&self, value: &dyn Recordable) -> Option<String>;

    fn get_internal_index(&self, value: &dyn Recordable) -> Option<usize>;

    fn type_id(&self) -> TypeId;
}

impl Eq for dyn ErasedProperty {}

impl PartialEq for dyn ErasedProperty {
    fn eq(&self, other: &Self) -> bool {
        ErasedProperty::type_id(self) == ErasedProperty::type_id(other)
            && self.name() == other.name()
    }
}

impl Hash for dyn ErasedProperty {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u128(
            unsafe { std::mem::transmute(ErasedProperty::type_id(self)) }
        );

        self.name().hash(state);

        self.possible_values().hash(state);
    }
}