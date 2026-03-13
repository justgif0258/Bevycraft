use crate::prelude::*;

pub struct MappedCommit<T: Recordable> {
    entries: RegistrationMap<T>,
}

impl<T: Recordable> Commit<T> for MappedCommit<T> {
    fn make() -> Self {
        Self { entries: RegistrationMap::default() }
    }

    #[inline]
    fn push(&mut self, key: RegistrationId, recordable: T) {
        assert!(!self.entries.contains_key(&key), "Tried registering with duplicate key");

        self.entries.insert(key, recordable);
    }

    #[inline]
    fn append(&mut self, entry: Entry<T>) {
        assert!(!self.entries.contains_key(entry.key()), "Tried registering with duplicate key");

        let taken = entry.take();

        self.entries.insert(taken.0, taken.1);
    }

    fn keys(&self) -> Vec<RegistrationId> {
        self.entries.keys()
            .cloned()
            .collect()
    }

    fn merge(&mut self, other: Self) {
        let entries = other.consume();

        entries.into_iter()
            .for_each(|entry| {
                self.append(entry);
            })
    }

    fn consume(self) -> Entries<T> {
        self.entries.into_iter()
            .map(|(key, val)| Entry::new(key, val))
            .collect()
    }
}