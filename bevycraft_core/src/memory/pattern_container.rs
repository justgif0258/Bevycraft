use std::{
    hash::{BuildHasher, Hash, Hasher},
    mem::{ManuallyDrop, transmute},
    num::NonZeroUsize,
    ops::Deref,
};

use bitvec::{field::BitField, vec::BitVec};
use hashbrown::{Equivalent, HashTable};
use rapidhash::fast::RandomState;

const SENTINEL: usize = usize::MAX;

pub struct PatternContainer<T, const N: usize, S = RandomState> {
    hasher: S,
    entries: HashTable<PatternEntry>,
    patterns: Vec<Slot<T>>,
    next_free: Option<usize>,
    container: BitVec,
    bit_len: BitLen,
}

impl<T, const N: usize> PatternContainer<T, N, RandomState> {
    #[inline]
    pub fn new() -> Self {
        Self {
            hasher: RandomState::new(),
            entries: HashTable::new(),
            patterns: Vec::new(),
            next_free: None,
            container: BitVec::with_capacity(N),
            bit_len: BitLen::default(),
        }
    }

    #[inline]
    pub fn with_bit_len(bit_len: usize) -> Self {
        Self {
            hasher: RandomState::new(),
            entries: HashTable::new(),
            patterns: Vec::new(),
            next_free: None,
            container: BitVec::with_capacity(N * bit_len),
            bit_len: BitLen::new(bit_len).unwrap(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            hasher: RandomState::new(),
            entries: HashTable::with_capacity(capacity),
            patterns: Vec::with_capacity(capacity),
            next_free: None,
            container: BitVec::with_capacity(N),
            bit_len: BitLen::default(),
        }
    }

    #[inline]
    pub fn with_bit_len_and_capacity(bit_len: usize, capacity: usize) -> Self {
        Self {
            hasher: RandomState::new(),
            entries: HashTable::with_capacity(capacity),
            patterns: Vec::with_capacity(capacity),
            next_free: None,
            container: BitVec::with_capacity(N * bit_len),
            bit_len: BitLen::new(bit_len).unwrap(),
        }
    }

    #[inline]
    pub const fn size(&self) -> usize {
        N
    }

    #[inline]
    pub fn entries(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn bit_len(&self) -> usize {
        *self.bit_len
    }
}

impl<T, const N: usize, S> PatternContainer<T, N, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    pub fn set_at(&mut self, index: usize, value: T) {
        let start = index * *self.bit_len;
        let end = start + *self.bit_len;

        let pattern = if let Some(pattern) = self.try_get_pattern(&value) {
            pattern
        } else {
            self.add_pattern(value)
        };

        self.container[start..end].store(pattern);
    }

    #[inline]
    fn try_get_pattern(&self, value: &T) -> Option<usize> {
        let hash = make_hash(&self.hasher, value);

        self.entries
            .find(hash, equivalent_pattern(&self.patterns, value))
            .map(|e| e.pattern)
    }

    #[inline]
    fn add_pattern(&mut self, value: T) -> usize {
        let hash = make_hash(&self.hasher, &value);

        let value = ManuallyDrop::new(value);

        let index = if let Some(index) = self.next_free() {
            self.patterns[index].value = value;

            index
        } else {
            let next = self.patterns.len();

            self.patterns.push(Slot { value });

            next
        };

        self.entries.insert_unique(
            hash,
            PatternEntry {
                pattern: index,
                counts: 1,
            },
            make_hash_from_pattern(&self.hasher, &self.patterns),
        );

        index
    }

    #[inline]
    fn remove_pattern(&mut self, pattern: usize) {
        let value: &T = unsafe { &self.patterns[pattern].value };

        let hash = make_hash(&self.hasher, value);

        if let Ok(entry) = self
            .entries
            .find_entry(hash, equivalent_pattern(&self.patterns, value))
        {
            let (removed, _) = entry.remove();

            self.drop_slot(removed.pattern);
        }
    }

    #[inline]
    fn next_free(&mut self) -> Option<usize> {
        if let Some(next) = self.next_free {
            let new = unsafe {
                let n = self.patterns[next].next;

                if n != SENTINEL { Some(n) } else { None }
            };

            self.next_free = new;

            return Some(next);
        }

        None
    }

    #[inline]
    fn drop_slot(&mut self, index: usize) {
        let last = self.next_free.unwrap_or(SENTINEL);

        let slot = &mut self.patterns[index];

        unsafe { ManuallyDrop::drop(&mut slot.value) }

        slot.next = last;
        self.next_free = Some(index);
    }
}

#[inline(always)]
fn equivalent_pattern<Q, T>(patterns: &[Slot<T>], v: &Q) -> impl Fn(&PatternEntry) -> bool
where
    Q: Equivalent<T> + ?Sized,
{
    move |entry| {
        let t = unsafe { &patterns[entry.pattern].value };

        v.equivalent(t)
    }
}

#[inline(always)]
fn make_hash_from_pattern<T, S>(
    hash_builder: &S,
    patterns: &[Slot<T>],
) -> impl Fn(&PatternEntry) -> u64
where
    T: Hash + Sized,
    S: BuildHasher,
{
    move |entry| {
        let t: &T = unsafe { &patterns[entry.pattern].value };

        make_hash(hash_builder, t)
    }
}

#[inline(always)]
fn make_hash<T, S>(hash_builder: &S, value: &T) -> u64
where
    T: Hash + ?Sized,
    S: BuildHasher,
{
    let mut hasher = hash_builder.build_hasher();

    value.hash(&mut hasher);

    hasher.finish()
}

#[inline(always)]
const fn required_bits(value: usize) -> usize {
    if value == 0 {
        return 1;
    }

    (usize::BITS - value.leading_zeros()) as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PatternEntry {
    pattern: usize,
    counts: usize,
}

impl PatternEntry {
    #[inline(always)]
    pub(crate) fn inc_counts(&mut self) {
        self.counts += 1;
    }

    #[inline(always)]
    pub(crate) fn dec_counts(&mut self) {
        self.counts -= 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BitLen(NonZeroUsize);

impl Deref for BitLen {
    type Target = usize;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.0 as *const NonZeroUsize as *const usize) }
    }
}

impl Default for BitLen {
    #[inline]
    fn default() -> Self {
        Self(NonZeroUsize::new(1).unwrap())
    }
}

impl BitLen {
    #[inline]
    pub fn new(len: usize) -> Option<Self> {
        unsafe { transmute(len) }
    }
}

union Slot<T> {
    value: std::mem::ManuallyDrop<T>,
    next: usize,
}
