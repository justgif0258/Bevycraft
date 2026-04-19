use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};
use bevy::prelude::{IVec3, Resource};
use voxelis::spatial::{VoxOpsRead, VoxOpsWrite, VoxTree};
use voxelis::{MaxDepth, VoxInterner};

const DEFAULT_MEMORY_BUDGET: usize = 256 * 1024 * 1024;

const DEFAULT_DEPTH: MaxDepth = MaxDepth::new(4);

pub struct SectionOctree {
    interner: WorldInterner,
    octree  : VoxTree,
}

impl SectionOctree {
    #[inline]
    pub fn new(interner: WorldInterner) -> Self {
        Self {
            interner,
            octree: VoxTree::new(DEFAULT_DEPTH),
        }
    }

    #[inline(always)]
    pub fn set(&mut self, position: impl Into<[i32; 3]>, block_id: u32) {
        let mut interner = self.interner.write().unwrap();

        let position = position.into();

        self.octree.set(
            &mut interner,
            position.into(),
            block_id + 1
        );
    }

    #[inline(always)]
    pub fn remove(&mut self, position: impl Into<[i32; 3]>) {
        let mut interner = self.interner.write().unwrap();

        let position = position.into();

        self.octree.set(
            &mut interner,
            position.into(),
            0
        );
    }

    #[inline(always)]
    pub fn get(&self, position: impl Into<[i32; 3]>) -> Option<u32> {
        let interner = self.interner.write().unwrap();

        let position = position.into();

        self.octree.get(
            &interner,
            position.into(),
        )
            .map(|v| v - 1)
    }
}

#[derive(Resource)]
pub struct WorldInterner {
    interner: Arc<RwLock<VoxInterner<u32>>>
}

impl Clone for WorldInterner {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            interner: self.interner.clone()
        }
    }
}

impl WorldInterner {
    #[inline]
    pub fn new() -> Self {
        Self {
            interner: Arc::new(RwLock::new(VoxInterner::with_memory_budget(DEFAULT_MEMORY_BUDGET)))
        }
    }

    #[inline]
    pub fn with_memory_budget(requested_budget: usize) -> Self {
        Self {
            interner: Arc::new(RwLock::new(VoxInterner::with_memory_budget(requested_budget)))
        }
    }

    #[inline(always)]
    fn write(&self) -> LockResult<RwLockWriteGuard<'_, VoxInterner<u32>>> {
        self.interner.write()
    }

    #[inline(always)]
    fn read(&self) -> LockResult<RwLockReadGuard<'_, VoxInterner<u32>>> {
        self.interner.read()
    }
}