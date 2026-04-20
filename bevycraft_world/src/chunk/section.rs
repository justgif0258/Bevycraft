use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;

pub(crate) const SECTION_UPPER_BOUND: IVec3 = IVec3::splat(SECTION_SIZE);

pub(crate) const SECTION_LOWER_BOUND: IVec3 = IVec3::ZERO;

pub(crate) const SECTION_SIZE: i32 = 16;

pub(crate) const SECTION_LEN: usize = 4096;

pub struct PalettedSection {
    blocks      : SectionArray<SECTION_LEN>,
    palette     : HashMap<u64, (u32, u32), NoOpHash>, // (Global Index -> Local Index | Ref counts)
    to_global   : Vec<usize>,                           // (Global Index | Ref counts)
    free_list   : Vec<u16>,
    needs_resize: bool,
}

impl PalettedSection {
    pub const EMPTY: u32 = 0u32;

    #[inline]
    pub fn new() -> Self {
        let mut refs = Vec::with_capacity(8);

        refs.push((usize::MAX));

        Self {
            blocks: SectionArray::ArrayU8(Box::new([0u8; SECTION_LEN])),
            palette: HashMap::with_hasher(NoOpHash),
            to_global: refs,
            free_list: Vec::with_capacity(8),
            needs_resize: false,
        }
    }

    #[inline]
    pub fn empty() -> Self {
        let mut refs = Vec::with_capacity(8);

        refs.push(usize::MAX);

        Self {
            blocks: SectionArray::Empty,
            palette: HashMap::with_hasher(NoOpHash),
            to_global: refs,
            free_list: Vec::with_capacity(8),
            needs_resize: false,
        }
    }

    #[inline(always)]
    pub fn get(&self, position: impl Into<IVec3>) -> Option<usize> {
        let local_index = self.blocks.get(map_to_flat_index(position));

        if local_index == Self::EMPTY {
            return None;
        }

        Some(self.to_global[local_index as usize])
    }

    #[inline(always)]
    pub fn set(&mut self, position: impl Into<IVec3>, global_index: usize) {
        let idx = map_to_flat_index(position);

        let local = self.get_or_insert_index(global_index);

        self.blocks.set(idx, local)
    }

    #[inline(always)]
    pub fn remove(&mut self, position: impl Into<IVec3>) -> Option<usize> {
        let idx = map_to_flat_index(position);

        let local = self.blocks.get(idx);

        if local == Self::EMPTY {
            return None;
        }

        let old_global = self.to_global[local as usize];

        self.blocks.set(idx, Self::EMPTY);

        let entry = self.palette.get_mut(&(old_global as u64)).unwrap();

        if entry.1 == 0 {
            self.free_list.push(local as u16);
            self.palette.remove(&(old_global as u64));

            self.needs_resize = true;
        }

        Some(old_global)
    }

    #[inline(always)]
    pub fn palette_len(&self) -> usize {
        self.palette.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.palette.is_empty()
    }

    #[inline(always)]
    pub const fn needs_resize(&self) -> bool {
        self.needs_resize
    }

    #[inline(always)]
    fn get_or_insert_index(&mut self, global_index: usize) -> u32 {
        if let Some(existing) = self.palette.get_mut(&(global_index as u64)) {
            existing.1 += 1;

            return existing.0;
        }

        if let Some(freed) = self.free_list.pop().map(|f| f as u32) {
            self.to_global[freed as usize] = global_index;
            self.palette.insert(global_index as u64, (freed, 1));

            return freed;
        }

        let next = self.to_global.len() as u32;

        self.to_global.push(global_index);
        self.palette.insert(global_index as u64, (next, 1));

        next
    }
}

pub enum SectionArray<const N: usize> {
    Empty,
    SingleValue(u32),
    ArrayU8(Box<[u8; N]>),
    ArrayU16(Box<[u16; N]>),
    ArrayU32(Box<[u32; N]>),
}

impl<const N: usize> SectionArray<N> {
    #[inline(always)]
    pub fn get(&self, index: usize) -> u32 {
        match self {
            SectionArray::Empty => 0,
            SectionArray::SingleValue(v) => *v,
            SectionArray::ArrayU8(b) => b[index] as u32,
            SectionArray::ArrayU16(b) => b[index] as u32,
            SectionArray::ArrayU32(b) => b[index],
        }
    }

    #[inline(always)]
    pub fn set(&mut self, index: usize, value: u32) {
        match self {
            SectionArray::Empty => (),
            SectionArray::SingleValue(v) => *v = value,
            SectionArray::ArrayU8(b) => b[index] = value as u8,
            SectionArray::ArrayU16(b) => b[index] = value as u16,
            SectionArray::ArrayU32(b) => b[index] = value,
        }
    }
}

#[inline(always)]
fn map_to_flat_index(position: impl Into<IVec3>) -> usize {
    let position = position.into();

    debug_assert!(
        position.cmplt(SECTION_UPPER_BOUND).all()
            && position.cmpge(SECTION_LOWER_BOUND).all(),
        "Tried indexing out of the section boundaries"
    );

    (position.x + (position.z * SECTION_SIZE) + (position.y * SECTION_SIZE * SECTION_SIZE)) as usize
}