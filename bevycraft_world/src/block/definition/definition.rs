use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use bevycraft_core::prelude::Recordable;

#[derive(Debug)]
pub struct Definition<T: Recordable> {
    name: &'static str,
    default: T,
    normalizer: Option<fn(T) -> T>,
}

impl<T: Recordable> Eq for Definition<T> {}

impl<T: Recordable> PartialEq for Definition<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(&self, &other)
    }
}

impl<T: Recordable> Hash for Definition<T> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64((self as *const _) as u64);
    }
}

impl<T: Recordable> Definition<T> {
    pub const fn new(name: &'static str, default: T) -> Self {
        Self { name, default, normalizer: None }
    }

    pub const fn with_normalizer(mut self, normalizer: fn(T) -> T) -> Self {
        self.normalizer = Some(normalizer);
        self
    }

    #[inline(always)]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    #[inline(always)]
    pub const fn default(&self) -> &T {
        &self.default
    }

    #[inline(always)]
    pub fn normalize(&self, value: T) -> T {
        if let Some(normalizer) = self.normalizer {
            return normalizer(value);
        }

        value
    }
}

impl<T: Recordable> ErasedDefinition for Definition<T> {
    #[inline(always)]
    fn name(&self) -> &'static str {
        self.name
    }
}

pub trait ErasedDefinition: Recordable {
    fn name(&self) -> &'static str;

    #[inline(always)]
    fn as_erased(&self) -> &dyn ErasedDefinition
    where
        Self: Sized
    {
        self
    }
}

impl Debug for dyn ErasedDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Eq for dyn ErasedDefinition {}

impl PartialEq for dyn ErasedDefinition {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(&self, &other)
    }
}

impl Hash for dyn ErasedDefinition {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(
            unsafe { *(self as *const _ as *const u64) }
        )
    }
}