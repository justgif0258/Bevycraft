use std::any::TypeId;

use crate::prelude::{AssetLocation, Registrable, RegistrationError, Registry};

pub trait ErasedRegistry: Send + Sync + 'static {
    fn erased_keys(&self) -> Box<dyn Iterator<Item = &AssetLocation> + '_>;

    fn contains_erased(&self, location: &AssetLocation) -> bool;

    fn get_erased_by_key(&self, location: &AssetLocation) -> Option<&dyn Registrable>;

    fn get_erased_by_idx(&self, index: usize) -> Option<&dyn Registrable>;

    fn erased_key_to_idx(&self, location: &AssetLocation) -> Option<usize>;

    fn erased_idx_to_key(&self, index: usize) -> Option<&AssetLocation>;

    fn erased_len(&self) -> usize;

    fn register_erased(
        &mut self,
        location: AssetLocation,
        value: Box<dyn Registrable>,
    ) -> Result<(), RegistrationError>;

    fn registry_id(&self) -> TypeId;

    #[inline(always)]
    fn erase_registry(&self) -> &dyn ErasedRegistry
    where
        Self: Sized,
    {
        self
    }
}

impl<R: Registry> ErasedRegistry for R {
    #[inline]
    fn erased_keys(&self) -> Box<dyn Iterator<Item = &AssetLocation> + '_> {
        Box::new(self.keys())
    }

    #[inline]
    fn contains_erased(&self, location: &AssetLocation) -> bool {
        self.contains_key(location)
    }

    #[inline]
    fn get_erased_by_key(&self, location: &AssetLocation) -> Option<&dyn Registrable> {
        self.get_by_key(location).map(|v| v as &dyn Registrable)
    }

    #[inline]
    fn get_erased_by_idx(&self, index: usize) -> Option<&dyn Registrable> {
        self.get_by_idx(index).map(|v| v as &dyn Registrable)
    }

    #[inline]
    fn erased_key_to_idx(&self, location: &AssetLocation) -> Option<usize> {
        self.key_to_idx(location)
    }

    #[inline]
    fn erased_idx_to_key(&self, index: usize) -> Option<&AssetLocation> {
        self.idx_to_key(index)
    }

    #[inline]
    fn erased_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn register_erased(
        &mut self,
        location: AssetLocation,
        value: Box<dyn Registrable>,
    ) -> Result<(), RegistrationError> {
        if let Ok(downcasted) = value.downcast::<R::Item>() {
            return self.register(location, *downcasted);
        }

        Err(RegistrationError::DowncastFailed)
    }

    #[inline(always)]
    fn registry_id(&self) -> TypeId {
        TypeId::of::<R>()
    }
}

impl dyn ErasedRegistry {
    #[inline(always)]
    pub fn downcast<R: Registry>(self: Box<Self>) -> Result<Box<R>, Box<Self>> {
        if self.is::<R>() {
            unsafe { Ok(self.downcast_unchecked()) }
        } else {
            Err(self)
        }
    }

    #[inline(always)]
    pub fn downcast_mut<R: Registry>(&mut self) -> Option<&mut R> {
        if self.is::<R>() {
            unsafe { Some(self.downcast_mut_unchecked::<R>()) }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn downcast_ref<R: Registry>(&self) -> Option<&R> {
        if self.is::<R>() {
            unsafe { Some(self.downcast_ref_unchecked::<R>()) }
        } else {
            None
        }
    }

    #[inline(always)]
    pub unsafe fn downcast_unchecked<R: Registry>(self: Box<Self>) -> Box<R> {
        unsafe {
            let raw = Box::into_raw(self);

            Box::from_raw(raw as *mut R)
        }
    }

    #[inline(always)]
    pub const unsafe fn downcast_mut_unchecked<R: Registry>(&mut self) -> &mut R {
        unsafe { &mut *(self as *mut dyn ErasedRegistry as *mut R) }
    }

    #[inline(always)]
    pub const unsafe fn downcast_ref_unchecked<R: Registry>(&self) -> &R {
        unsafe { &*(self as *const dyn ErasedRegistry as *const R) }
    }

    #[inline(always)]
    fn is<R: Registry>(&self) -> bool {
        self.registry_id() == TypeId::of::<R>()
    }
}
