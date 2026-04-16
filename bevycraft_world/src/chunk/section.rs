use std::alloc::{alloc_zeroed, Layout};
use std::ptr::slice_from_raw_parts_mut;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use frozen_collections::Len;

pub struct Section {
    palette : RcPalette,
    blocks  : Box<[u16]>,
}

impl Section {
    const SECTION_LEN: usize = 4096;

    pub(crate) const SECTION_SIZE: UVec3 = UVec3::new(16, 16, 16);

    #[inline]
    pub fn allocate() -> Self {
        let blocks = unsafe {
            let layout = Layout::array::<u16>(Self::SECTION_LEN)
                .unwrap();

            let ptr = alloc_zeroed(layout);

            let fat_ptr = slice_from_raw_parts_mut(ptr as *mut u16, Self::SECTION_LEN);

            Box::from_raw(fat_ptr)
        };

        Self {
            palette: RcPalette::new(),
            blocks
        }
    }

    #[inline]
    pub fn from_allocated(alloc: Box<[u16]>) -> Option<Self> {
        if alloc.len() != Self::SECTION_LEN {
            return None;
        }

        Some({
            Self {
                palette: RcPalette::new(),
                blocks: alloc
            }
        })
    }

    #[inline(always)]
    pub fn set_at(&mut self, pos: impl Into<UVec3>, global_idx: u32) {
        let idx = Self::map_to_flat_index(pos.into());

        let loc_id = self.palette.inc_ref_or_add(global_idx);

        self.blocks[idx] = loc_id;
    }

    #[inline(always)]
    pub fn get_at(&self, pos: impl Into<UVec3>) -> u32 {
        let idx = Self::map_to_flat_index(pos.into());

        self.palette.get_global_idx(self.blocks[idx]).unwrap()
    }
    
    #[inline(always)]
    pub fn recycle(self) -> Box<[u16]> {
        self.blocks
    }

    #[inline(always)]
    fn map_to_flat_index(pos: UVec3) -> usize {
        debug_assert!(pos.cmplt(Self::SECTION_SIZE).all(), "Tried indexing out of the section boundaries");

        (pos.x + (pos.z * Self::SECTION_SIZE.x) + (pos.y * Self::SECTION_SIZE.x * Self::SECTION_SIZE.z)) as usize
    }
}

pub struct RcPalette {
    entries     : Vec<(u32, u16)>,              // (Global Index | Ref counts)
    lookup      : HashMap<u32, u16, NoOpHash>,  // (Global Index | Local Index)
    free_list   : Vec<u16>,                     // (Local Indexes)
    needs_resize: bool,
}

impl RcPalette {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            lookup: HashMap::with_hasher(NoOpHash),
            free_list: Vec::new(),
            needs_resize: false,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            lookup: HashMap::with_capacity_and_hasher(capacity, NoOpHash),
            free_list: Vec::with_capacity(capacity),
            needs_resize: false,
        }
    }

    #[inline(always)]
    pub fn get_global_idx(&self, local_idx: u16) -> Option<u32> {
        self.entries.get(local_idx as usize)
            .map(|(idx, _)| *idx)
    }

    #[inline(always)]
    #[must_use]
    pub fn inc_ref_or_add(&mut self, global_idx: u32) -> u16 {
        if let Some(&entry) = self.lookup.get(&global_idx) {
            self.inc_ref(entry);

            return entry;
        }

        if let Some(free_idx) = self.free_list.pop() {
            self.entries[free_idx as usize] = (global_idx, 1);

            self.lookup.insert(global_idx, free_idx);

            if self.free_list.len() == 0 {
                self.needs_resize = false;
            }

            return free_idx;
        }

        let new_index = self.entries.len() as u16;

        self.entries.push((global_idx, 1));
        self.lookup.insert(global_idx, new_index);

        new_index
    }

    #[inline(always)]
    pub fn dec_ref_or_remove(&mut self, local_idx: u16) {
        self.dec_ref(local_idx);

        let entry = &mut self.entries[local_idx as usize];

        if entry.1 == 0 {
            self.lookup.remove(&entry.0);

            self.free_list.push(local_idx);
        }
    }

    #[inline(always)]
    fn inc_ref(&mut self, local_idx: u16) {
        self.entries[local_idx as usize].1 += 1;
    }

    #[inline(always)]
    fn dec_ref(&mut self, local_idx: u16) {
        let rc = &mut self.entries[local_idx as usize];

        rc.1 = rc.1.saturating_sub(1);
    }
}