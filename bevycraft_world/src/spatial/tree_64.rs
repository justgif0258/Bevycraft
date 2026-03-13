use std::num::NonZeroUsize;
use std::slice::from_raw_parts;
use bevy::prelude::UVec3;
use crate::prelude::{Morton3D, Node64};

pub struct Tree64<T: Default + Send + Sync> {
    root        : Node64,
    node_pool   : Vec<Node64>,
    leaf_pool   : Vec<T>,
    depth       : NonZeroUsize,
}

impl<T: Default + Send + Sync> Tree64<T> {
    pub const MAX_DEPTH: usize = 16;

    #[inline]
    pub const fn new(depth: usize) -> Self {
        debug_assert!(depth <= Self::MAX_DEPTH, "Maximum supported depth is 16");

        let depth = NonZeroUsize::new(depth)
            .expect("Depth should be non-zero");

        Self {
            root: Node64::EMPTY,
            node_pool: Vec::new(),
            leaf_pool: Vec::new(),
            depth,
        }
    }

    pub fn set_at_depth_recursive(&mut self, depth: usize, pos: UVec3, leaf: T) {
        debug_assert!(depth <= self.depth());

        let mut current = self.get_root();

        let morton = Morton3D::encode(pos);

        for level in 1..self.depth() {
            let cluster_idx = self.get_morton_idx(level, &morton);

            if let Some(cluster) = self.get_cluster(current) {
                current = &cluster[cluster_idx];
                break;
            }

            todo!()
        }
    }

    #[inline]
    const fn get_root(&self) -> &Node64 {
        &self.root
    }

    #[inline]
    const fn get_brick(&self, node: &Node64) -> Option<&[T]> {
        if node.is_brick() && node.has_children() {
            return Some(unsafe { self.get_brick_unchecked(node) })
        }

        None
    }

    #[inline]
    const fn get_cluster(&self, node: &Node64) -> Option<&[Node64]> {
        if node.is_cluster() && node.has_children() {
            return Some(unsafe { self.get_cluster_unchecked(node) })
        }

        None
    }

    #[inline]
    const unsafe fn get_brick_unchecked(&self, node: &Node64) -> &[T] {
        unsafe {
            from_raw_parts(
                self.leaf_pool
                    .as_ptr()
                    .add(node.get_child_ptr()),
                Node64::MAX_CHILDREN,
            )
        }
    }

    #[inline]
    const unsafe fn get_cluster_unchecked(&self, node: &Node64) -> &[Node64] {
        unsafe {
            from_raw_parts(
                self.node_pool
                    .as_ptr()
                    .add(node.get_child_ptr()),
                Node64::MAX_CHILDREN,
            )
        }
    }

    #[inline]
    pub const fn depth(&self) -> usize {
        self.depth.get()
    }

    #[inline]
    pub fn get_morton_idx(&self, current_depth: usize, morton: &Morton3D) -> usize {
        debug_assert!(current_depth <= self.depth(), "Current depth should be within the Tree's defined depth");

        let shift = (self.depth() - current_depth) * 6;

        ((morton >> shift) & 0x3F).raw() as usize
    }
}