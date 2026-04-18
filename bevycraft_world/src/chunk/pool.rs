use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::{Arc, LockResult, RwLock, RwLockWriteGuard};
use std::time::Instant;
use bevy::prelude::*;

#[derive(Resource)]
pub struct GlobalSectionPool {
    inner   : Arc<SectionPool>,
    _marker : PhantomData<SectionPool>,
}

impl GlobalSectionPool {
    #[inline]
    pub fn new(delta: f64) -> Self {
        assert!(delta > 0.0, "Delta must be greater than 0");

        Self {
            inner: Arc::new(SectionPool {
                garbage: RwLock::new(VecDeque::new()),
                delta_decay: delta,
            }),
            _marker : PhantomData,
        }
    }

    #[inline]
    pub fn recycle(&self, memory: Box<[u16]>, ) {
        let mut garbage = self.acquire_garbage().unwrap();

        let expiration = Self::now_secs_f64() + self.inner.delta_decay;

        garbage.push_back((memory, expiration));
    }

    #[inline]
    pub fn alloc(&self) -> Box<[u16]> {
        let mut garbage = self.acquire_garbage().unwrap();

        if let Some((section, _)) = garbage.pop_front() {
            return section;
        }

        Box::new([0u16; 4096])
    }

    #[inline]
    pub fn alloc_zeroed(&self) -> Box<[u16]> {
        let mut garbage = self.acquire_garbage().unwrap();

        if let Some((mut section, _)) = garbage.pop_front() {
            section.fill(0);

            return section;
        }

        Box::new([0u16; 4096])
    }

    #[inline]
    pub fn collect_garbage(&self) {
        let mut garbage = self.acquire_garbage().unwrap();
        
        let current_time = Self::now_secs_f64();

        while let Some(&(_, expiration)) = garbage.front() {
            if current_time >= expiration {
                garbage.pop_front();
            } else {
                break;
            }
        }
    }

    #[inline(always)]
    fn acquire_garbage(&self) -> LockResult<RwLockWriteGuard<'_, VecDeque<(Box<[u16]>, f64)>>> {
        self.inner.garbage.write()
    }
    
    #[inline(always)]
    fn now_secs_f64() -> f64 {
        Instant::now().elapsed().as_secs_f64()
    }
}

impl Clone for GlobalSectionPool {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker : PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct SectionPool {
    garbage     : RwLock<VecDeque<(Box<[u16]>, f64)>>,
    delta_decay : f64,
}

impl SectionPool {
    #[inline]
    pub const fn new(delta: f64) -> Self {
        assert!(delta > 0.0, "Delta must be greater than 0");
        
        Self {
            garbage: RwLock::new(VecDeque::new()),
            delta_decay: delta,
        }
    }

    #[inline]
    pub fn new_global(delta: f64) -> Arc<Self> {
        assert!(delta > 0.0, "Delta must be greater than 0");

        Arc::new(Self {
            garbage: RwLock::new(VecDeque::new()),
            delta_decay: delta,
        })
    }

    #[inline]
    pub fn recycle(&self, memory: Box<[u16]>, current_time: f64) {
        let mut garbage = self.acquire_garbage().unwrap();

        let expiration = current_time + self.delta_decay;

        garbage.push_back((memory, expiration));
    }

    #[inline]
    pub fn alloc(&self) -> Box<[u16]> {
        let mut garbage = self.acquire_garbage().unwrap();

        if let Some((section, _)) = garbage.pop_front() {
            return section;
        }

        Box::new([0u16; 4096])
    }

    pub fn alloc_zeroed(&self) -> Box<[u16]> {
        let mut garbage = self.acquire_garbage().unwrap();

        if let Some((mut section, _)) = garbage.pop_front() {
            section.fill(0);

            return section;
        }

        Box::new([0u16; 4096])
    }

    #[inline]
    pub fn collect_garbage(&self, current_time: f64) {
        let mut garbage = self.acquire_garbage().unwrap();

        while let Some(&(_, expiration)) = garbage.front() {
            if current_time >= expiration {
                garbage.pop_front();
            } else {
                break;
            }
        }
    }

    #[inline(always)]
    fn acquire_garbage(&self) -> LockResult<RwLockWriteGuard<'_, VecDeque<(Box<[u16]>, f64)>>> {
        self.garbage.write()
    }
}
