use std::hash::Hash;
use std::marker::PhantomData;

pub struct CompiledRegistry<T: Send + Sync + 'static> {
    namespace: &'static str,
    entries: phf::OrderedMap<&'static str, T>,
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> CompiledRegistry<T> {
    pub const fn new(
        namespace: &'static str,
        entries: phf::OrderedMap<&'static str, T>,
    ) -> Self {
        Self { namespace, entries, _marker: PhantomData }
    }

    #[inline]
    pub fn get_by_path(&self, path: &str) -> Option<&T> {
        self.entries.get(path)
    }

    #[inline]
    pub fn get_by_id(&self, index: usize) -> Option<&T> {
        self.entries.index(index).map(|(_, v)| v)
    }

    #[inline]
    pub fn path_to_id(&self, path: &str) -> Option<usize> {
        self.entries.get_index(path)
    }

    #[inline]
    pub fn id_to_path(&self, id: usize) -> Option<&'static str> {
        self.entries.index(id).map(|(k, _)| *k)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &T)> {
        self.entries.into_iter().map(|(k, v)| (*k, v))
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<T: Send + Sync + 'static> Hash for CompiledRegistry<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
    }
}