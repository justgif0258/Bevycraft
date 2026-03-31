use crate::prelude::*;

pub trait Commit<T: Recordable>: Default {
    fn push(&mut self, key: AssetLocation, recordable: T);

    fn append(&mut self, entry: Entry<T>);

    fn keys(&self) -> Vec<AssetLocation>;

    fn merge(&mut self, other: Self);

    /// Consumes the commit into a raw entry list.
    fn consume(self) -> Entries<T>;
}
