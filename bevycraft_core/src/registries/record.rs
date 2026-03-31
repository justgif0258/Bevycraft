use {
    crate::prelude::*,
    std::{any::TypeId, sync::Arc},
};

pub trait Record<T: Recordable> {
    fn get_by_key(&self, key: &AssetLocation) -> Option<&T>;

    fn get_by_id(&self, id: usize) -> Option<&T>;

    fn idx_to_key(&self, id: usize) -> Option<&AssetLocation>;

    fn key_to_idx(&self, key: &AssetLocation) -> Option<usize>;

    fn keys(&self) -> Vec<&AssetLocation>;

    fn len(&self) -> usize;
}

/// # Recordable
/// A trait that must be implemented for all types that can be stored in a [`Record`].
///
/// # Safety
/// Implementors must ensure that the type is [`Send`] and [`Sync`], and that the [`TypeId`] is stable across compilations.
pub unsafe trait Recordable: Send + Sync + 'static {
    fn type_id(&self) -> TypeId;
}

unsafe impl<T> Recordable for T
where
    T: Send + Sync + 'static,
{
    #[inline(always)]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl dyn Recordable {
    #[inline(always)]
    pub fn downcast<T: Recordable>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            unsafe { return Ok(self.downcast_unchecked()) }
        }

        Err(self)
    }

    #[inline(always)]
    pub fn downcast_arc<T: Recordable>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>> {
        if self.is::<T>() {
            unsafe { return Ok(self.downcast_arc_unchecked()) }
        }

        Err(self)
    }

    #[inline(always)]
    pub fn downcast_ref<T: Recordable>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { return Some(self.downcast_ref_unchecked()) }
        }

        None
    }

    #[inline(always)]
    pub unsafe fn downcast_unchecked<T: Recordable>(self: Box<Self>) -> Box<T> {
        unsafe {
            let raw = Box::into_raw(self);

            Box::from_raw(raw as *mut T)
        }
    }

    #[inline(always)]
    pub unsafe fn downcast_arc_unchecked<T: Recordable>(self: Arc<Self>) -> Arc<T> {
        unsafe {
            let raw = Arc::into_raw(self);

            Arc::from_raw(raw as *const T)
        }
    }

    #[inline(always)]
    pub unsafe fn downcast_ref_unchecked<T: Recordable>(&self) -> &T {
        unsafe { &*(self as *const Self as *const T) }
    }

    #[inline(always)]
    fn is<T: Recordable>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }
}
