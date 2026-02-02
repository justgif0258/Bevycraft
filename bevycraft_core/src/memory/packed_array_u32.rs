use std::alloc::*;
use std::fmt::{Debug, Formatter};
use std::num::{NonZeroUsize};
use std::ptr::*;

const BYTE_BITS: usize = u8::BITS as usize;

/// # Packed Index Array
/// Fast, memory-safe, efficient array with dynamically bit-sized indices.
/// The bit-size of each index is based on how many bits is needed for the
/// largest index to be accurately interpreted.
pub struct PackedArrayU32 {
    buffer: Option<NonNull<u8>>,
    layout: Layout,
    bits: NonZeroUsize,
    size: usize,
}

impl PackedArrayU32 {
    const MAX_BITS: usize = u32::BITS as usize;

    const INITIAL_BITS: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };

    pub fn new(size: usize) -> Self {
        let alloc_size = size.div_ceil(BYTE_BITS);

        let layout = unsafe {
            Layout::from_size_align_unchecked(
                alloc_size,
                align_of::<u8>(),
            )
        };

        let buffer = NonNull::new(unsafe { alloc_zeroed(layout) })
            .expect("Failed to allocate memory");

        Self {
            buffer: Some(buffer),
            layout,
            bits: Self::INITIAL_BITS,
            size,
        }
    }

    #[inline]
    pub const fn zeroed(size: usize) -> Self {
        Self {
            buffer: None,
            layout: unsafe {
                Layout::from_size_align_unchecked(
                    size.div_ceil(BYTE_BITS),
                    align_of::<u8>()
                )
            },
            bits: Self::INITIAL_BITS,
            size,
        }
    }

    pub fn with_bit_length(size: usize, bits: usize) -> Self {
        debug_assert!(bits <= Self::MAX_BITS, "Length must not exceed 32-bits");

        let alloc_size = (bits * size).div_ceil(BYTE_BITS);

        let bits = NonZeroUsize::new(bits)
            .expect("Bit length must not be zero");

        let layout = unsafe {
            Layout::from_size_align_unchecked(
                alloc_size,
                align_of::<u8>(),
            )
        };

        let buffer = NonNull::new(unsafe { alloc_zeroed(layout) })
            .expect("Failed to allocate memory");

        Self {
            buffer: Some(buffer),
            layout,
            bits,
            size,
        }
    }

    #[inline]
    pub const fn zeroed_with_bit_length(size: usize, bits: usize) -> Self {
        debug_assert!(bits <= Self::MAX_BITS, "Length must not exceed 32-bits");

        Self {
            buffer: None,
            layout: unsafe {
                Layout::from_size_align_unchecked(
                    (bits * size).div_ceil(BYTE_BITS),
                    align_of::<u8>()
                )
            },
            bits: NonZeroUsize::new(bits)
                .expect("Bit length must not be zero"),
            size,
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> u32 {
        self.read_value(index)
            .expect("Failed to read value")
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: u32) {
        self.write_value(index, value)
            .expect("Failed to write value");
    }

    #[inline]
    const fn read_value(&self, index: usize) -> Result<u32, &'static str> {
        debug_assert!(index < self.size, "Index out of bounds");

        match self.buffer {
            None => Err("Tried reading value on None"),
            Some(_) => {
                Ok(unsafe { self.get_unchecked(index) })
            }
        }
    }

    #[inline]
    fn write_value(&mut self, index: usize, value: u32) -> Result<(), &'static str> {
        debug_assert!(index < self.size, "Index out of bounds");

        match self.buffer {
            None => Err("Tried writing value on None"),
            Some(_) => {
                let req = required_bits(value);

                if req > self.bit_length() {
                    self.grow_bits_by(req - self.bit_length());
                }

                unsafe { self.set_unchecked(index, value); }

                Ok(())
            }
        }
    }

    #[inline]
    pub fn grow_bits_by_powf2(&mut self) {
        self.resize_bits(1isize << self.bit_length());
    }

    #[inline]
    pub fn grow_bits_by(&mut self, amount: usize) {
        self.resize_bits(amount as isize);
    }

    fn resize_bits(&mut self, resize_factor: isize) {
        let old_bits = self.bit_length();
        let new_bits = (old_bits as isize + resize_factor).max(1) as usize;

        debug_assert!(new_bits <= Self::MAX_BITS, "Bit Length must not exceed 32-bits");

        let bytes_len = (new_bits * self.size).div_ceil(BYTE_BITS);

        let new_layout = unsafe {
            Layout::from_size_align_unchecked(
                bytes_len,
                align_of::<u8>()
            )
        };
        
        let new_buffer = NonNull::new(unsafe { alloc_zeroed(new_layout) })
            .expect("Failed to reallocate memory");

        if let Some (old_buffer) = self.buffer.replace(new_buffer) {
            let mut src_index = 0usize;
            let mut dst_index = 0usize;

            for _ in 0..self.size {
                unsafe {
                    let value = Self::read_bits_from_buffer(
                        old_buffer.as_ptr(),
                        src_index,
                        old_bits
                    );

                    Self::write_bits_to_buffer(
                        new_buffer.as_ptr(),
                        dst_index,
                        new_bits,
                        value
                    );
                }

                src_index += old_bits;
                dst_index += new_bits;
            }

            unsafe { dealloc(old_buffer.as_ptr(), self.layout) };
        }

        self.layout = new_layout;
        self.bits = unsafe { NonZeroUsize::new_unchecked(new_bits) };
    }

    /// **Undefined behaviour warning!**
    ///
    /// Function may produce undefined behaviour as no bounds are checked.
    #[inline]
    pub const unsafe fn get_unchecked(&self, index: usize) -> u32 {
        let bit_length = self.bit_length();

        Self::read_bits_from_buffer(
            self.buffer.unwrap_unchecked().as_ptr(),
            bit_length.unchecked_mul(index),
            bit_length
        )
    }

    /// **Undefined behaviour warning!**
    ///
    /// Function may produce undefined behaviour as no bounds are checked and may lead to possible data corruption.
    #[inline]
    pub const unsafe fn set_unchecked(&mut self, index: usize, value: u32) {
        let bit_length = self.bit_length();

        Self::write_bits_to_buffer(
            self.buffer.unwrap_unchecked().as_ptr(),
            bit_length.unchecked_mul(index),
            bit_length,
            value
        )
    }

    #[inline(always)]
    const unsafe fn read_bits_from_buffer(
        buffer: *const u8,
        bit_index: usize,
        n_bits: usize,
    ) -> u32 {
        let mut row = (buffer.add(bit_index >> 3) as *const u64).read_unaligned();

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
        let ptr = buffer.add(bit_index >> 3) as *mut u64;

        let mut row = ptr.read_unaligned();

        let shift = bit_index & 7;

        row &= !(mask(n_bits) << shift);
        row |= (bits as u64) << shift;

        ptr.write_unaligned(row);
    }

    pub fn allocate(&mut self) -> Result<(), &'static str> {
        match self.buffer {
            None => {
                let buffer = NonNull::new(unsafe { alloc_zeroed(self.layout) })
                    .expect("Failed to allocate memory");

                self.buffer = Some(buffer);

                Ok(())
            }
            Some(_) => Err("Tried overwriting allocated buffer"),
        }
    }

    pub fn deallocate(&mut self) -> Result<(), &'static str> {
        match self.buffer {
            None => Err("Tried deallocating non-existing buffer"),
            Some(buffer) => unsafe {
                dealloc(buffer.as_ptr(), self.layout);

                Ok(())
            }
        }
    }

    #[inline]
    pub const fn bit_length(&self) -> usize {
        self.bits.get()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_none()
    }

    #[inline]
    pub const fn allocated_memory(&self) -> usize {
        if self.buffer.is_some() { self.layout.size() } else { 0usize }
    }
}

impl Drop for PackedArrayU32 {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            unsafe { dealloc(buffer.as_ptr(), self.layout) }
        }
    }
}

impl Debug for PackedArrayU32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PackedArrayU32(Allocated Memory: {}B, Bit Length: {}, Size: {})", self.allocated_memory(), self.bit_length(), self.size)
    }
}

const fn mask(len: usize) -> u64 {
    (1u64 << len) - 1
}

#[inline]
const fn required_bits(value: u32) -> usize {
    (u32::BITS - value.leading_zeros()) as usize
}