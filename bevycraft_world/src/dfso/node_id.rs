use crate::dfso::MAX_DEPTH;

const CHILD_PTR_MASK: u32 = 0xFFFFFF;
const CHILDREN_MASK: u32 = 0xFF000000;

const CHILDREN_SHIFT: usize = 24;

pub struct NodeId(u32);

impl AsRef<NodeId> for NodeId {
    #[inline]
    fn as_ref(&self) -> &NodeId {
        &self
    }
}

impl AsMut<NodeId> for NodeId {
    #[inline]
    fn as_mut(&mut self) -> &mut NodeId {
        self
    }
}

impl From<u32> for NodeId {
    #[inline]
    fn from(id: u32) -> Self {
        NodeId(id)
    }
}

impl From<NodeId> for u32 {
    #[inline]
    fn from(id: NodeId) -> Self {
        id.0
    }
}

impl NodeId {
    const MAX: usize = 0xFFFFFF;

    pub const EMPTY: NodeId = NodeId(0);

    #[inline]
    const fn new(index: usize, children: u8) -> Self {
        debug_assert!(index <= Self::MAX, "Node index must not exceed NodeId::MAX");

        Self((index as u32) | ((children as u32) << CHILDREN_SHIFT))
    }

    #[inline]
    pub const fn new_branch(index: usize, children: u8) -> Self {
        Self::new(index, children)
    }

    #[inline]
    pub const fn new_leaf(index: usize) -> Self {
        Self::new(index, 0xFF)
    }

    #[inline]
    pub const fn idx(&self) -> usize {
        (self.0 & CHILD_PTR_MASK) as usize
    }

    #[inline]
    pub const fn has_children(&self) -> bool {
        (self.0 & CHILDREN_MASK) != 0
    }

    #[inline]
    pub const fn has_child(&self, index: usize) -> bool {
        debug_assert!(index < 8, "A branch can only have 8 children");

        ((self.0 >> CHILDREN_SHIFT) & (0x1 << index)) != 0
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == Self::EMPTY.0
    }
}

pub struct Depth(u8);

impl Depth {
    #[inline]
    pub const fn new(depth: usize) -> Self {
        debug_assert!(depth <= MAX_DEPTH, "Maximum allowed depth is 7");

        Self(depth as u8)
    }

    #[inline]
    pub const fn depth(&self) -> usize {
        self.0 as usize
    }
}

pub struct OctreeTraversal(u8, u8);

impl OctreeTraversal {
    #[inline]
    const fn new(depth: Depth, starting_depth: usize) -> Self {
        debug_assert!(starting_depth < depth.depth(), "Cannot start at the current or lower depth");

        OctreeTraversal(depth.0, starting_depth as u8)
    }

    #[inline]
    pub const fn new_traversal(depth: Depth) -> Self {
        Self::new(depth, 0)
    }

    #[inline]
    pub const fn new_at(depth: Depth, starting_depth: usize) -> Self {
        Self::new(depth, starting_depth)
    }
}