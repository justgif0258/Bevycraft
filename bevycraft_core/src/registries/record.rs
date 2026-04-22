use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use {
    crate::prelude::*,
    bevy::ecs::resource::Resource,
    std::{any::TypeId, sync::Arc},
};

/// # Record
/// A trait for storing an immutable collection of [`Recordable`] values.
/// The underlying data structure should not allow mutation after construction.
/// By the nature of the trait, implementors are free to prioritize read performance over write performance.
pub trait Record: Resource {
    type Value: Recordable;

    type Index: Copy + Eq;

    fn finish<C>(commit: C) -> Self
    where
        C: Commit<Value = Self::Value>;
    
    fn get_by_key(&self, key: &AssetLocation) -> Option<&Self::Value>;
    
    fn get_by_idx(&self, index: Self::Index) -> Option<&Self::Value>;

    fn key_to_idx(&self, key: &AssetLocation) -> Option<Self::Index>;

    fn idx_to_key(&self, index: Self::Index) -> Option<&AssetLocation>;
    
    fn iter(&self) -> impl Iterator<Item = &(AssetLocation, Self::Value)>;

    fn iter_keys(&self) -> impl Iterator<Item = &AssetLocation>;

    fn len(&self) -> usize;
}

/// # Recordable
/// A trait that must be implemented for all types that can be stored in a [`Record`].
///
/// # Safety
/// Implementors must ensure that the type is [`Send`] and [`Sync`], and that the [`TypeId`] is stable across compilations.
pub unsafe trait Recordable: Send + Sync + 'static {
    fn as_recordable(&self) -> &dyn Recordable;

    fn type_id(&self) -> TypeId;
}

unsafe impl<T> Recordable for T
where
    T: Send + Sync + 'static,
{
    #[inline(always)]
    fn as_recordable(&self) -> &dyn Recordable {
        self
    }

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

impl Debug for dyn Recordable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Recordable")
            .finish_non_exhaustive()
    }
}

impl Eq for dyn Recordable {}

impl PartialEq for dyn Recordable {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(&self, &other)
    }
}

impl Hash for dyn Recordable {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u128(
            unsafe { std::mem::transmute(self) },
        );
    }
}