use {
    bitvec::{field::BitField, vec::BitVec},
    hashbrown::{Equivalent, HashTable},
    rapidhash::fast::RandomState,
    std::{
        cmp::Ordering,
        fmt::{Debug, Formatter},
        hash::{BuildHasher, Hash, Hasher},
        mem::{transmute, ManuallyDrop},
        num::NonZeroUsize,
        ops::Neg,
    },
};

const SENTINEL: usize = usize::MAX;

pub struct PatternContainer<T, const N: usize, S = RandomState> {
    hasher: S,
    entries: HashTable<Pattern>,
    patterns: Vec<Slot<T>>,
    next_free: Option<usize>,
    container: BitVec,
    bit_cap: BitCapacity,
}

impl<T, const N: usize, S> Debug for PatternContainer<T, N, S>
where
    T: Debug,
    S: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatternContainer")
            .field("hasher", &self.hasher)
            .field("patterns", &self.entries)
            .field("bit_cap", &self.bit_cap)
            .finish()
    }
}

impl<T, const N: usize, S> Clone for PatternContainer<T, N, S>
where
    T: Clone,
    S: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            hasher: self.hasher.clone(),
            entries: self.entries.clone(),
            patterns: self.patterns.clone(),
            next_free: self.next_free,
            container: self.container.clone(),
            bit_cap: self.bit_cap,
        }
    }
}

impl<T, const N: usize> PatternContainer<T, N, RandomState>
where
    T: Eq + Hash,
{
    #[inline]
    pub fn new(initial: T) -> Self {
        Self::new_with(initial, RandomState::new(), 1, 1)
    }

    #[inline]
    pub fn with_bit_capacity(initial: T, bit_cap: usize) -> Self {
        Self::new_with(initial, RandomState::new(), bit_cap, 1)
    }

    #[inline]
    pub fn with_capacity(initial: T, capacity: usize) -> Self {
        Self::new_with(initial, RandomState::new(), 1, capacity)
    }

    #[inline]
    pub fn with_bit_cap_and_capacity(initial: T, bit_cap: usize, capacity: usize) -> Self {
        Self::new_with(initial, RandomState::new(), bit_cap, capacity)
    }
}

impl<T, const N: usize, S> PatternContainer<T, N, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    pub fn new_with(initial: T, hasher: S, bit_cap: usize, capacity: usize) -> Self {
        let mut entries: HashTable<Pattern> = HashTable::with_capacity(capacity);
        let mut patterns: Vec<Slot<T>> = Vec::with_capacity(capacity);

        entries.insert_unique(
            make_hash(&hasher, &initial),
            Pattern {
                index: 0,
                counts: N,
            },
            make_hash_from_pattern(&hasher, &patterns),
        );

        patterns.push(Slot {
            value: ManuallyDrop::new(initial),
        });

        Self {
            hasher,
            entries,
            patterns,
            next_free: None,
            container: BitVec::repeat(false, N * bit_cap),
            bit_cap: BitCapacity::new(bit_cap).unwrap(),
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        let bit_len = self.bit_cap.get();
        let start = index * bit_len;
        let end = start + bit_len;

        let pattern = self.container[start..end].load::<usize>();

        self.patterns
            .get(pattern)
            .map(|s| unsafe { &s.value as &T })
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        self.decay_pattern_at_index(index);

        let pattern = self.get_or_insert_pattern(value);

        let bit_len = self.bit_cap.get();
        let start = index * bit_len;
        let end = start + bit_len;

        self.container[start..end].store(pattern);
    }

    #[inline]
    fn get_or_insert_pattern(&mut self, value: T) -> usize {
        if let Some(pattern) = self.get_pattern_mut(&value) {
            pattern.counts += 1;

            return pattern.index;
        }

        self.add_pattern(value)
    }

    #[inline]
    fn decay_pattern_at_index(&mut self, index: usize) {
        let bit_len = self.bit_cap.get();
        let start = index * bit_len;
        let end = start + bit_len;

        let index = self.container[start..end].load::<usize>();

        let pattern: &T = unsafe { &self.patterns[index].value };

        if let Ok(mut occupied) = self.entries.find_entry(
            make_hash(&self.hasher, pattern),
            equivalent_pattern(&self.patterns, pattern),
        ) {
            occupied.get_mut().counts -= 1;

            if occupied.get().counts == 0 {
                let (removed, _) = occupied.remove();

                self.drop_slot(removed.index);
            }
        }
    }

    #[inline]
    fn get_pattern_mut(&mut self, value: &T) -> Option<&mut Pattern> {
        let hash = make_hash(&self.hasher, value);

        self.entries
            .find_mut(hash, equivalent_pattern(&self.patterns, value))
    }

    #[inline]
    fn add_pattern(&mut self, value: T) -> usize {
        let hash = make_hash(&self.hasher, &value);

        let value = ManuallyDrop::new(value);

        let pattern = if let Some(index) = self.pop_free() {
            self.patterns[index].value = value;

            index
        } else {
            let next = self.patterns.len();

            if required_bits(next) > self.bit_cap.get() {
                let amount = required_bits(next) - self.bit_cap.get();

                self.grow_bit_capacity(amount);
            }

            self.patterns.push(Slot { value });

            next
        };

        self.entries.insert_unique(
            hash,
            Pattern {
                index: pattern,
                counts: 1,
            },
            make_hash_from_pattern(&self.hasher, &self.patterns),
        );

        pattern
    }

    #[inline]
    fn pop_free(&mut self) -> Option<usize> {
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

impl<T, const N: usize, S> PatternContainer<T, N, S> {
    #[inline]
    pub const fn iter(&self) -> PatternIter<'_, T, N, S> {
        PatternIter {
            container: &self,
            index: 0,
            cursor: 0,
        }
    }

    #[inline]
    pub fn try_compress(&mut self) -> bool {
        let active = self.entries.len();

        let new_cap = BitCapacity::for_count(active);

        if new_cap >= self.bit_cap {
            return false;
        }

        let mut remap = vec![0usize; self.patterns.len()];
        let mut new_idx = 0usize;

        for entry in self.entries.iter_mut() {
            remap[entry.index] = new_idx;
            entry.index = new_idx;
            new_idx += 1;
        }

        for old_idx in 0..self.patterns.len() {
            let new_idx = remap[old_idx];
            if new_idx != old_idx {
                unsafe {
                    let value = ManuallyDrop::take(&mut self.patterns[old_idx].value);

                    self.patterns[new_idx].value = ManuallyDrop::new(value);
                }
            }
        }

        self.patterns.truncate(active);
        self.next_free = None;

        let src_cap = self.bit_cap.get();
        let dst_cap = new_cap.get();

        let mut new_container = BitVec::repeat(false, N * dst_cap);

        let mut src_next = 0usize;
        let mut dst_next = 0usize;

        for _ in 0..N {
            let extracted = self.container[src_next..src_next + src_cap].load::<usize>();

            let actual_idx = remap[extracted];

            new_container[dst_next..dst_next + dst_cap].store(actual_idx);

            src_next += src_cap;
            dst_next += dst_cap;
        }

        self.container = new_container;
        self.bit_cap = new_cap;

        true
    }

    #[inline]
    fn grow_bit_capacity(&mut self, amount: usize) {
        self.resize_bit_capacity(amount as isize)
    }

    #[inline]
    #[allow(dead_code)]
    fn shrink_bit_capacity(&mut self, amount: usize) {
        self.resize_bit_capacity((amount as isize).neg());
    }

    fn resize_bit_capacity(&mut self, factor: isize) {
        if factor == 0 {
            return;
        }

        let new_cap = self.bit_cap.factor_by(factor).unwrap();

        let src_cap = self.bit_cap.get();
        let dst_cap = new_cap.get();

        let mut new_container = BitVec::repeat(false, N * dst_cap);

        let mut src_next = 0usize;
        let mut dst_next = 0usize;

        for _ in 0..N {
            let extracted = self.container[src_next..src_next + src_cap].load::<usize>();

            new_container[dst_next..dst_next + dst_cap].store(extracted);

            src_next += src_cap;
            dst_next += dst_cap;
        }

        self.container = new_container;
        self.bit_cap = new_cap;
    }

    #[inline]
    pub fn as_single(&self) -> Option<&T> {
        if self.entries.len() > 1 {
            return None;
        }

        let pattern = self.entries.iter().next()?;

        unsafe { Some(&self.patterns[pattern.index].value) }
    }

    #[inline]
    pub const fn size(&self) -> usize {
        N
    }

    #[inline]
    pub fn active_entries(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub const fn bit_capacity(&self) -> usize {
        self.bit_cap.get()
    }
}

pub struct PatternIter<'a, T, const N: usize, S = RandomState> {
    container: &'a PatternContainer<T, N, S>,
    index: usize,
    cursor: usize,
}

impl<'a, T, const N: usize, S> ExactSizeIterator for PatternIter<'a, T, N, S> {
    #[inline]
    fn len(&self) -> usize {
        N
    }
}

impl<'a, T, const N: usize, S> Iterator for PatternIter<'a, T, N, S> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= N {
            return None;
        }

        let bit_cap = self.container.bit_cap.get();
        let cursor = self.cursor;
        let tail = cursor + bit_cap;

        let idx = self.container.container[cursor..tail].load::<usize>();

        self.index += 1;
        self.cursor = tail;

        unsafe { Some(&self.container.patterns[idx].value) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = N - self.index;
        (remaining, Some(remaining))
    }
}

#[inline(always)]
fn equivalent_pattern<Q, T>(patterns: &[Slot<T>], v: &Q) -> impl Fn(&Pattern) -> bool
where
    Q: Equivalent<T> + ?Sized,
{
    move |entry| {
        let t = unsafe { &patterns[entry.index].value };

        v.equivalent(t)
    }
}

#[inline(always)]
fn make_hash_from_pattern<T, S>(hash_builder: &S, patterns: &[Slot<T>]) -> impl Fn(&Pattern) -> u64
where
    T: Hash + Sized,
    S: BuildHasher,
{
    move |entry| {
        let t: &T = unsafe { &patterns[entry.index].value };

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
pub(crate) struct Pattern {
    index: usize,
    counts: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BitCapacity(NonZeroUsize);

impl Default for BitCapacity {
    #[inline]
    fn default() -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(1) })
    }
}

impl BitCapacity {
    pub(crate) const MAX: usize = usize::BITS as usize;

    #[inline]
    pub(crate) const fn new(value: usize) -> Option<Self> {
        assert!(value <= Self::MAX);

        unsafe { transmute(value) }
    }

    #[inline]
    pub(crate) const fn for_count(count: usize) -> Self {
        if count == 0 {
            return Self::new(1).unwrap();
        }

        Self::new(required_bits(count - 1)).unwrap()
    }

    #[inline]
    pub(crate) fn factor_by(self, factor: isize) -> Option<Self> {
        let new = self.get() as isize + factor;

        if new < 1 || new > Self::MAX as isize {
            return None;
        }

        unsafe { transmute(new) }
    }

    #[inline]
    #[allow(unused)]
    pub(crate) const fn checked_add(self, amount: usize) -> Option<Self> {
        let new = self.get() + amount;

        if new > Self::MAX {
            return None;
        }

        unsafe { transmute(new) }
    }

    #[inline]
    #[allow(unused)]
    pub(crate) const fn checked_sub(self, amount: usize) -> Option<Self> {
        let current = self.get();

        if current <= amount {
            return None;
        }

        unsafe { transmute(current - amount) }
    }

    #[inline]
    pub(crate) const fn get(self) -> usize {
        self.0.get()
    }
}

union Slot<T> {
    value: ManuallyDrop<T>,
    next: usize,
}

impl<T> Copy for Slot<T> where T: Copy {}

impl<T> Clone for Slot<T>
where
    T: Clone,
{
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            Slot {
                value: ManuallyDrop::clone(&self.value),
            }
        }
    }
}

impl<T> Ord for Slot<T>
where
    T: Ord,
{
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.value.cmp(&other.value) }
    }
}

impl<T> PartialOrd for Slot<T>
where
    T: PartialOrd,
{
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unsafe { self.value.partial_cmp(&other.value) }
    }
}

impl<T> Eq for Slot<T> where T: Eq {}

impl<T> PartialEq for Slot<T>
where
    T: PartialEq,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.value.eq(&other.value) }
    }
}
