use {
    std::{
        any::TypeId,
        sync::Arc,
    },
    boomphf::Mphf,
    crate::prelude::*,
};

#[derive(Debug)]
pub struct Record<T: ?Sized + Recordable> {
    m_hasher: Mphf<RegistrationId>,
    entries : Box<[Entry<T>]>,
}

impl<T: Recordable> Record<T> {
    pub const BASE: f64 = 3.3f64;

    pub fn new<C: Commit<T>>(commit: C) -> Self {
        let keys = commit.keys();

        let m_hasher = Self::gen_phf(keys);

        let entries = Self::gen_boxed_entries(&m_hasher, commit);

        Self {
            m_hasher,
            entries,
        }
    }

    #[inline]
    pub fn get_by_key(&self, key: &RegistrationId) -> Option<&T> {
        let idx = self.m_hasher.try_hash(key)?;

        self.entries.get(idx as usize)
            .and_then(|entry| {
                if entry.key() == key {
                    return Some(entry.val())
                }

                None
            })
    }

    #[inline]
    pub fn get_by_id(&self, index: usize) -> Option<&T> {
        self.entries.get(index)
            .map(|entry| entry.val())
    }

    #[inline]
    pub fn idx_to_key(&self, index: usize) -> Option<&RegistrationId> {
        self.entries.get(index)
            .map(|entry| entry.key())
    }

    #[inline]
    pub fn key_to_idx(&self, key: &RegistrationId) -> Option<usize> {
        self.m_hasher.try_hash(key)
            .map(|idx| idx as usize)
    }

    #[inline]
    pub fn keys(&self) -> Vec<&RegistrationId> {
        self.entries
            .iter()
            .map(|entry| entry.key())
            .collect()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    fn gen_boxed_entries<C: Commit<T>>(phf: &Mphf<RegistrationId>, commit: C) -> Box<[Entry<T>]> {
        let entries = commit.consume();

        let mut boxed = Box::<[Entry<T>]>::new_uninit_slice(entries.len());

        entries.into_iter()
            .for_each(|entry| {
                let idx = phf.hash(entry.key()) as usize;

                boxed[idx].write(entry);
            });

        unsafe { boxed.assume_init() }
    }

    fn gen_phf(keys: Vec<RegistrationId>) -> Mphf<RegistrationId> {
        Mphf::new(Self::BASE, keys.as_slice())
    }
}

pub unsafe trait Recordable: Send + Sync + 'static {
    fn type_id(&self) -> TypeId;
}

unsafe impl<T> Recordable for T
where
    T: Send + Sync + 'static
{
    #[inline]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl dyn Recordable {

    #[inline]
    pub fn downcast<T: Recordable>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            unsafe { return Ok(self.downcast_unchecked()) }
        }

        Err(self)
    }

    #[inline]
    pub fn downcast_arc<T: Recordable>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>> {
        if self.is::<T>() {
            unsafe { return Ok(self.downcast_arc_unchecked()) }
        }

        Err(self)
    }

    #[inline]
    pub fn downcast_ref<T: Recordable>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { return Some(self.downcast_ref_unchecked()) }
        }

        None
    }

    #[inline]
    pub unsafe fn downcast_unchecked<T: Recordable>(self: Box<Self>) -> Box<T> {
        unsafe {
            let raw = Box::into_raw(self);

            Box::from_raw(raw as *mut T)
        }
    }

    #[inline]
    pub unsafe fn downcast_arc_unchecked<T: Recordable>(self: Arc<Self>) -> Arc<T> {
        unsafe {
            let raw = Arc::into_raw(self);

            Arc::from_raw(raw as *const T)
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: Recordable>(&self) -> &T {
        unsafe {
            &*(self as *const Self as *const T)
        }
    }

    #[inline]
    fn is<T: Recordable>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }
}