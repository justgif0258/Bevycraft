use std::alloc::*;
use std::fmt::{Debug, Formatter};
use std::num::NonZeroUsize;
use std::ops::Add;
use std::ptr::*;

/// ## Packed Index Array
/// Fast, memory-safe and efficient array with dynamically bit-sized indices.
/// The bit-size of each index is based on how many bits is needed for the
/// largest index to be represented.
pub struct PackedArrayU32 {
    buffer: NonNull<u8>,
    layout: Layout,
    bits: NonZeroUsize,
    size: usize,
}

impl PackedArrayU32 {
    pub const MAX_BITS: usize = u32::BITS as usize;

    const INITIAL_BITS: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };

    pub fn new(size: usize) -> Self {
        let layout = unsafe {
            Layout::from_size_align_unchecked(alloc_size(size, 1usize), align_of::<u8>())
        };

        let buffer =
            NonNull::new(unsafe { alloc_zeroed(layout) }).expect("Failed to allocate memory");

        Self {
            buffer,
            layout,
            bits: Self::INITIAL_BITS,
            size,
        }
    }

    #[inline]
    pub const fn zeroed() -> Self {
        Self {
            buffer: NonNull::dangling(),
            layout: unsafe { Layout::from_size_align_unchecked(0usize, align_of::<u8>()) },
            bits: Self::INITIAL_BITS,
            size: 0,
        }
    }

    pub fn with_bit_length(size: usize, bits: usize) -> Self {
        debug_assert!(bits <= Self::MAX_BITS, "Bit length must not exceed 32-bits");

        let bits = NonZeroUsize::new(bits).expect("Bit length must be non-zero");

        let layout = unsafe {
            Layout::from_size_align_unchecked(alloc_size(size, bits.get()), align_of::<u8>())
        };

        let buffer =
            NonNull::new(unsafe { alloc_zeroed(layout) }).expect("Failed to allocate memory");

        Self {
            buffer,
            layout,
            bits,
            size,
        }
    }

    #[inline]
    pub const fn zeroed_with_bit_length(bits: usize) -> Self {
        debug_assert!(bits <= Self::MAX_BITS, "Bit length must not exceed 32-bits");

        Self {
            buffer: NonNull::dangling(),
            layout: unsafe { Layout::from_size_align_unchecked(0usize, align_of::<u8>()) },
            bits: NonZeroUsize::new(bits).expect("Bit length must be non-zero"),
            size: 0,
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> u32 {
        debug_assert!(index < self.size, "Index out of bounds");

        unsafe { self.get_unchecked(index) }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: u32) {
        debug_assert!(index < self.size, "Index out of bounds");

        debug_assert!(
            required_bits(value) <= self.bit_length(),
            "Value bit length was too large to fit into the current array",
        );

        unsafe { self.set_unchecked(index, value) }
    }

    #[inline]
    pub fn grow_bits(&mut self) {
        debug_assert!(
            (self.bit_length() << 1) <= Self::MAX_BITS,
            "Unable to resize bits. New bit length exceeded the 32-bit limit"
        );

        unsafe { self.resize_bits(self.bit_length() as isize) }
    }

    #[inline]
    pub fn grow_bits_by(&mut self, amount: usize) {
        debug_assert!(
            self.bit_length().add(amount) <= Self::MAX_BITS,
            "Unable to resize bits. New bit length exceeded the 32-bit limit"
        );

        unsafe { self.resize_bits(amount as isize) }
    }

    #[inline]
    pub fn shrink_bits_by(&mut self, amount: usize) {
        unsafe { self.resize_bits(-(amount as isize)) }
    }

    pub unsafe fn resize_bits(&mut self, resize_factor: isize) {
        let old_bits = self.bit_length();
        let new_bits = (old_bits as isize + resize_factor).max(0) as usize;

        let bits = NonZeroUsize::new(new_bits).expect("Resized bit length must be non-zero");

        unsafe {
            if self.size > 0 {
                let new_layout = Layout::from_size_align_unchecked(
                    alloc_size(self.size, new_bits),
                    align_of::<u8>(),
                );

                let new_buffer =
                    NonNull::new(alloc_zeroed(new_layout)).expect("Failed to reallocate memory");

                let mut src_index = 0usize;
                let mut dst_index = 0usize;

                for _ in 0..self.size {
                    let value =
                        Self::read_bits_from_buffer(self.buffer.as_ptr(), src_index, old_bits);

                    Self::write_bits_to_buffer(new_buffer.as_ptr(), dst_index, new_bits, value);

                    src_index += old_bits;
                    dst_index += new_bits;
                }

                dealloc(self.buffer.as_ptr(), self.layout);

                self.buffer = new_buffer;
                self.layout = new_layout;
            }
        }

        self.bits = bits;
    }

    /// **Undefined behaviour warning!**
    ///
    /// Function may produce undefined behaviour as no bounds are checked.
    #[inline]
    pub const unsafe fn get_unchecked(&self, index: usize) -> u32 {
        let bit_length = self.bit_length();

        unsafe {
            Self::read_bits_from_buffer(
                self.buffer.as_ptr(),
                bit_length.unchecked_mul(index),
                bit_length,
            )
        }
    }

    /// **Undefined behaviour warning!**
    ///
    /// Function may produce undefined behaviour as no bounds are checked and may lead to possible data corruption.
    #[inline]
    pub const unsafe fn set_unchecked(&mut self, index: usize, value: u32) {
        let bit_length = self.bit_length();

        unsafe {
            Self::write_bits_to_buffer(
                self.buffer.as_ptr(),
                bit_length.unchecked_mul(index),
                bit_length,
                value,
            )
        }
    }

    #[inline(always)]
    const unsafe fn read_bits_from_buffer(
        buffer: *const u8,
        bit_index: usize,
        n_bits: usize,
    ) -> u32 {
        let mut row = unsafe { (buffer.add(bit_index >> 3) as *const u64).read_unaligned() };

        row >>= (bit_index & 7);
        row &= mask(n_bits);

        row as u32
    }

    #[inline(always)]
    const unsafe fn write_bits_to_buffer(
        buffer: *mut u8,
        bit_index: usize,
        n_bits: usize,
        bits: u32,
    ) {
        unsafe {
            let ptr = buffer.add(bit_index >> 3) as *mut u64;

            let mut row = ptr.read_unaligned();

            let shift = bit_index & 7;

            row &= !(mask(n_bits) << shift);
            row |= (bits as u64) << shift;

            ptr.write_unaligned(row);
        }
    }

    pub fn allocate(&mut self, size: usize) {
        debug_assert!(self.size == 0, "Cannot overwrite allocated buffer");

        unsafe {
            let layout = Layout::from_size_align_unchecked(
                alloc_size(size, self.bit_length()),
                align_of::<u8>(),
            );

            self.buffer = NonNull::new(alloc_zeroed(layout)).expect("Failed to allocate memory");

            self.layout = layout;
        }
    }

    pub fn deallocate(&mut self) {
        debug_assert!(self.size > 0, "Tried deallocating dangling reference");

        unsafe {
            dealloc(self.buffer.as_ptr(), self.layout);

            self.buffer = NonNull::dangling();

            self.layout = Layout::from_size_align_unchecked(0usize, align_of::<u8>());

            self.size = 0usize;
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub const fn bit_length(&self) -> usize {
        self.bits.get()
    }

    #[inline]
    pub const fn allocated_memory(&self) -> usize {
        if !self.is_empty() {
            self.layout.size()
        } else {
            0usize
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl Drop for PackedArrayU32 {
    fn drop(&mut self) {
        if self.size > 0 {
            unsafe { dealloc(self.buffer.as_ptr(), self.layout) }
        }
    }
}

impl Debug for PackedArrayU32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedArrayU32")
            .field("Allocated memory (B)", &self.allocated_memory())
            .field("Bit length", &self.bit_length())
            .field("Size", &self.size)
            .finish()
    }
}

#[inline]
const fn mask(len: usize) -> u64 {
    (1u64 << len) - 1
}

#[inline]
const fn required_bits(value: u32) -> usize {
    (u32::BITS - value.leading_zeros()) as usize
}

#[inline]
const fn alloc_size(size: usize, bits: usize) -> usize {
    (size * bits).div_ceil(u8::BITS as usize)
}
