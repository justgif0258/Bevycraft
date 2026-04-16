use std::collections::VecDeque;
use bevy::prelude::*;

#[derive(Resource)]
pub struct SectionPool {
    garbage     : VecDeque<(Box<[u16]>, f64)>,
    delta_decay : f64,
}

impl SectionPool {
    #[inline]
    pub const fn new(delta: f64) -> Self {
        assert!(delta > 0.0, "Delta must be greater than 0");
        
        Self {
            garbage: VecDeque::new(),
            delta_decay: delta,
        }
    }

    #[inline]
    pub fn recycle(&mut self, garbage: Box<[u16]>, current_time: f64) {
        let expiration = current_time + self.delta_decay;

        self.garbage.push_back((garbage, expiration));
    }

    #[inline]
    pub fn alloc(&mut self) -> Box<[u16]> {
        if let Some((section, _)) = self.garbage.pop_front() {
            return section;
        }

        Box::new([0u16; 4096])
    }

    pub fn alloc_zeroed(&mut self) -> Box<[u16]> {
        if let Some((mut section, _)) = self.garbage.pop_front() {
            section.fill(0);

            return section;
        }

        Box::new([0u16; 4096])
    }

    #[inline]
    pub fn collect_garbage(&mut self, current_time: f64) {
        while let Some(&(_, expiration)) = self.garbage.front() {
            if current_time >= expiration {
                self.garbage.pop_front();
            } else {
                break;
            }
        }
    }
}
