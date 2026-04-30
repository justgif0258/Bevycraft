use std::{
    any::TypeId,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::prelude::AssetLocation;

pub trait Registry: Send + Sync + 'static {
    type Item: Registrable;

    fn keys(&self) -> impl Iterator<Item = &AssetLocation>;

    fn contains_key(&self, location: &AssetLocation) -> bool;

    fn get_by_key(&self, location: &AssetLocation) -> Option<&Self::Item>;

    fn get_by_idx(&self, index: usize) -> Option<&Self::Item>;

    fn key_to_idx(&self, location: &AssetLocation) -> Option<usize>;

    fn idx_to_key(&self, index: usize) -> Option<&AssetLocation>;

    fn len(&self) -> usize;

    fn register(
        &mut self,
        location: AssetLocation,
        value: Self::Item,
    ) -> Result<(), RegistrationError>;
}

#[derive(Debug)]
pub enum RegistrationError {
    DuplicateKey,
    DowncastFailed,
    Custom(String),
}

impl std::error::Error for RegistrationError {}

impl std::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistrationError::DuplicateKey => f.write_str("Attempted to write on duplicated key"),
            RegistrationError::DowncastFailed => f.write_str("Failed to downcast registration"),
            RegistrationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

/// # Registrable
/// A trait that must be implemented for all types that can be stored in a [`Registry`].
///
/// # Safety
/// Implementors must ensure that the type is [`Send`] and [`Sync`], and that the [`TypeId`] is stable across compilations.
pub unsafe trait Registrable: Send + Sync + 'static {
    fn as_registrable(&self) -> &dyn Registrable;

    fn type_id(&self) -> TypeId;
}

unsafe impl<T> Registrable for T
where
    T: Send + Sync + 'static,
{
    #[inline(always)]
    fn as_registrable(&self) -> &dyn Registrable {
        self
    }

    #[inline(always)]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl dyn Registrable {
    #[inline(always)]
    pub fn downcast<T: Registrable>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            unsafe { return Ok(self.downcast_unchecked()) }
        }

        Err(self)
    }

    #[inline(always)]
    pub fn downcast_arc<T: Registrable>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>> {
        if self.is::<T>() {
            unsafe { return Ok(self.downcast_arc_unchecked()) }
        }

        Err(self)
    }

    #[inline(always)]
    pub fn downcast_ref<T: Registrable>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { return Some(self.downcast_ref_unchecked()) }
        }

        None
    }

    #[inline(always)]
    pub unsafe fn downcast_unchecked<T: Registrable>(self: Box<Self>) -> Box<T> {
        unsafe {
            let raw = Box::into_raw(self);

            Box::from_raw(raw as *mut T)
        }
    }

    #[inline(always)]
    pub unsafe fn downcast_arc_unchecked<T: Registrable>(self: Arc<Self>) -> Arc<T> {
        unsafe {
            let raw = Arc::into_raw(self);

            Arc::from_raw(raw as *const T)
        }
    }

    #[inline(always)]
    pub unsafe fn downcast_ref_unchecked<T: Registrable>(&self) -> &T {
        unsafe { &*(self as *const Self as *const T) }
    }

    #[inline(always)]
    fn is<T: Registrable>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }
}

impl Debug for dyn Registrable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Recordable").finish_non_exhaustive()
    }
}

impl Eq for dyn Registrable {}

impl PartialEq for dyn Registrable {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(&self, &other)
    }
}

impl Hash for dyn Registrable {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u128(unsafe { std::mem::transmute(self) });
    }
}
