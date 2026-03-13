const BRICK_SHIFT: u32 = 31;

const BRICK_MASK : u32 = 0x80000000;
const CHILD_MASK : u32 = 0x7FFFFFFF;

#[repr(C, packed(4))]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Node64 {
    child_ptr   : u32,
    child_mask  : u64,
}

impl Node64 {
    pub const MAX: u32 = i32::MAX as u32;

    pub const MAX_CHILDREN: usize = 64;

    pub const EMPTY: Self = Self { child_ptr: 0u32, child_mask: 0u64 };

    #[inline]
    pub const fn size() -> usize {
        size_of::<Self>()
    }

    #[inline]
    pub const fn align() -> usize {
        align_of::<Self>()
    }

    #[inline]
    const fn new(is_brick: bool, child_ptr: u32, child_mask: u64) -> Self {
        assert!(child_ptr <= Self::MAX, "A child pointer has overflown (MAX = 2_147_483_647)");

        Self {
            child_ptr: ((is_brick as u32) << BRICK_SHIFT) & child_ptr,
            child_mask,
        }
    }

    #[inline]
    pub const fn new_brick(child_ptr: u32, child_mask: u64) -> Self {
        Self::new(true, child_ptr, child_mask)
    }

    #[inline]
    pub const fn new_cluster(child_ptr: u32, child_mask: u64) -> Self {
        Self::new(false, child_ptr, child_mask)
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.child_mask == 0x0
    }

    #[inline]
    pub const fn is_brick(&self) -> bool {
        (self.child_ptr & BRICK_MASK) != 0x0
    }

    #[inline]
    pub const fn is_cluster(&self) -> bool {
        (self.child_ptr & BRICK_MASK) == 0x0
    }

    #[inline]
    pub const fn set_child_ptr(&mut self, child_ptr: u32) {
        assert!(child_ptr <= Self::MAX, "A child pointer has overflown (MAX = 2_147_483_647)");

        self.child_ptr &= BRICK_MASK;
        self.child_ptr |= child_ptr;
    }

    #[inline]
    pub const fn get_child_ptr(&self) -> usize {
        (self.child_ptr & CHILD_MASK) as usize
    }

    #[inline]
    pub const fn set_child_bit(&mut self, index: usize, value: bool) {
        debug_assert!(index < Self::MAX_CHILDREN, "A node can only have 64 children");

        self.child_mask &= (value as u64) << index;
    }

    #[inline]
    pub const fn has_children(&self) -> bool {
        self.child_mask != 0x0
    }

    #[inline]
    pub const fn has_child_at(&self, index: usize) -> bool {
        debug_assert!(index < Self::MAX_CHILDREN, "A node can only have 64 children");

        (self.child_mask & (0x1 << index)) != 0x0
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const Self {
        self.child_ptr as *const _
    }

    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut Self {
        self.child_ptr as *mut _
    }
}