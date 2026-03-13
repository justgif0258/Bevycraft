use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use {
    std::{
        slice::Iter,
        vec::IntoIter,
    },
    crate::prelude::*,
};

#[derive(Debug)]
pub struct Entries<T: ?Sized + Recordable>(Vec<Entry<T>>);

impl<T: ?Sized + Recordable> IntoIterator for Entries<T> {
    type Item = Entry<T>;
    type IntoIter = IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: ?Sized + Recordable> FromIterator<Entry<T>> for Entries<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item=Entry<T>>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl<T: ?Sized + Recordable> Entries<T> {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn add(&mut self, key: RegistrationId, val: T)
    where
        T: Sized + Recordable,
    {
        assert!(!self.contains(&key), "Found duplicate key '{}'", &key);

        self.0.push(Entry::new(key, val));
    }

    #[inline]
    pub fn add_boxed(&mut self, key: RegistrationId, val: Box<T>) {
        assert!(!self.contains(&key), "Found duplicate key '{}'", &key);

        self.0.push(Entry::with_boxed(key, val));
    }

    #[inline]
    pub fn append(&mut self, entry: Entry<T>) {
        assert!(!self.contains(&entry.key()), "Found duplicate key '{}'", entry.key());

        self.0.push(entry);
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Entry<T>> {
        self.0.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn contains(&self, key: &RegistrationId) -> bool {
        self.0.iter().any(|e| e == key)
    }
}

#[derive(Debug)]
pub struct Entry<T: ?Sized + Recordable> {
    key: RegistrationId,
    val: Box<T>,
}

impl<T: ?Sized + Recordable> Hash for Entry<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.key.hash_u64())
    }
}

impl<T: ?Sized + Recordable> Ord for Entry<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

impl<T: ?Sized + Recordable> PartialOrd for Entry<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T: ?Sized + Recordable> PartialOrd<RegistrationId> for Entry<T> {
    #[inline]
    fn partial_cmp(&self, other: &RegistrationId) -> Option<Ordering> {
        self.key.partial_cmp(other)
    }
}

impl<T: ?Sized + Recordable> Eq for Entry<T> {}

impl<T: ?Sized + Recordable> PartialEq for Entry<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<T: ?Sized + Recordable> PartialEq<RegistrationId> for Entry<T> {
    #[inline]
    fn eq(&self, other: &RegistrationId) -> bool {
        self.key.eq(other)
    }
}

impl<T: ?Sized + Recordable> Entry<T> {
    #[inline]
    pub fn new(key: RegistrationId, val: T) -> Self
    where
        T: Sized + Recordable,
    {
        Self { key, val: Box::new(val) }
    }

    pub fn with_boxed(key: RegistrationId, val: Box<T>) -> Self {
        Self { key, val }
    }

    #[inline]
    pub fn key(&self) -> &RegistrationId {
        &self.key
    }

    #[inline]
    pub fn val(&self) -> &T {
        &self.val
    }

    #[inline]
    pub fn take(self) -> (RegistrationId, T)
    where
        T: Sized + Recordable,
    {
        (self.key, *self.val)
    }

    pub fn take_boxed(self) -> (RegistrationId, Box<T>) {
        (self.key, self.val)
    }
}