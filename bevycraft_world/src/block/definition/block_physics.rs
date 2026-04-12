use bevy::math::bounding::{Aabb3d, IntersectsVolume};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

pub struct BlockPhysics {
    shapes: Vec<Aabb3d>,
}

impl BlockPhysics {
    #[inline(always)]
    pub fn does_intersect(&self, other: Aabb3d) -> bool {
        self.shapes
            .par_iter()
            .any(|shape| shape.intersects(&other))
    }
}