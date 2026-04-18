use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;

pub(crate) const SECTION_SIZE: u32 = 16;

pub(crate) const SECTION_LEN: usize = 4096;

pub struct PalettedSection {
    blocks      : SectionArray<SECTION_LEN>,
    palette     : HashMap<u64, u32, NoOpHash>,  // (Global Index -> Local Index)
    refs        : Vec<(u32, u32)>,              // (Global Index | Ref counts)
    free_list   : Vec<u16>,
    needs_resize: bool,
}

impl PalettedSection {
    pub const EMPTY: u32 = 0u32;

    #[inline]
    pub fn new() -> Self {
        let mut refs = Vec::with_capacity(8);

        refs.push((u32::MAX, u32::MAX));

        Self {
            blocks: SectionArray::ArrayU8(Box::new([0u8; SECTION_LEN])),
            palette: HashMap::with_hasher(NoOpHash),
            refs,
            free_list: Vec::with_capacity(8),
            needs_resize: false,
        }
    }

    #[inline]
    pub fn empty() -> Self {
        let mut refs = Vec::with_capacity(8);

        refs.push((u32::MAX, u32::MAX));

        Self {
            blocks: SectionArray::Empty,
            palette: HashMap::with_hasher(NoOpHash),
            refs,
            free_list: Vec::with_capacity(8),
            needs_resize: false,
        }
    }

    #[inline(always)]
    pub fn get(&self, position: impl Into<UVec3>) -> Option<u32> {
        let local_index = self.blocks.get(map_to_flat_index(position));

        if local_index == Self::EMPTY {
            return None;
        }

        Some(self.refs[local_index as usize].0)
    }

    #[inline(always)]
    pub fn set(&mut self, position: impl Into<UVec3>, global_index: u32) {
        let idx = map_to_flat_index(position);

        let local = self.global_to_local_index(global_index);

        self.inc_ref(local);

        self.blocks.set(idx, local)
    }

    #[inline(always)]
    pub fn remove(&mut self, position: impl Into<UVec3>) -> Option<u32> {
        let idx = map_to_flat_index(position);

        let local = self.blocks.get(idx);

        if local == 0 {
            return None;
        }

        let old_global = self.refs[local as usize].0;

        self.blocks.set(idx, 0);

        self.dec_ref(local);

        if self.refs[local as usize].1 == 0 {
            self.free_list.push(local as u16);

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
    fn global_to_local_index(&mut self, global_idx: u32) -> u32 {
        if let Some(&loc) = self.palette.get(&(global_idx as u64)) {
            return loc;
        }

        if let Some(loc) = self.free_list
            .pop()
            .map(|v| v as u32)
        {
            self.palette.insert(global_idx as u64, loc);

            return loc;
        }

        let loc = self.refs.len() as u32;

        self.refs.push((global_idx, 0));

        self.palette.insert(global_idx as u64, loc);

        loc
    }

    #[inline(always)]
    fn inc_ref(&mut self, local_idx: u32) {
        self.refs[local_idx as usize].1 += 1;
    }

    #[inline(always)]
    fn dec_ref(&mut self, local_idx: u32) {
        self.refs[local_idx as usize].1 -= 1;
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
fn map_to_flat_index(position: impl Into<UVec3>) -> usize {
    let position = position.into();

    debug_assert!(position.cmplt(UVec3::splat(SECTION_SIZE)).all(), "Tried indexing out of the section boundaries");

    (position.x + (position.z * SECTION_SIZE) + (position.y * SECTION_SIZE * SECTION_SIZE)) as usize
}