use crate::prelude::*;

pub trait Commit<T: ?Sized + Recordable> {
    fn make() -> Self;
    
    fn push(&mut self, key: RegistrationId, recordable: T);

    fn append(&mut self, entry: Entry<T>);

    fn keys(&self) -> Vec<RegistrationId>;

    fn merge(&mut self, other: Self);

    /// Consumes the commit into a raw entry list.
    fn consume(self) -> Entries<T>;
}