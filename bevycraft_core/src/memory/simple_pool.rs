use std::alloc::{alloc_zeroed, dealloc, handle_alloc_error, Layout};
use std::mem;
use std::ptr::NonNull;

pub struct SimplePool<T> {
    memory: NonNull<T>,
    layout: Layout,
    size: usize,
    next: usize,
}

unsafe impl<T> Send for SimplePool<T> {}
unsafe impl<T> Sync for SimplePool<T> {}

impl<T> SimplePool<T> {
    const fn block_size() -> usize {
        size_of::<T>()
    }

    const fn align() -> usize {
        align_of::<T>()
    }

    pub fn new(capacity: usize) -> SimplePool<T> {
        let size = Self::block_size() * capacity;
        let align = Self::align();

        let layout = Layout::from_size_align(size, align).expect("Invalid layout");

        let memory = unsafe {
            let ptr = alloc_zeroed(layout) as *mut T;

            if ptr.is_null() {
                handle_alloc_error(layout);
            }

            NonNull::new_unchecked(ptr)
        };

        Self {
            memory,
            layout,
            size: capacity,
            next: 0,
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> &T {
        unsafe { self.memory.add(index).as_ref() }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        unsafe { self.memory.add(index).as_mut() }
    }

    pub fn allocate(&mut self, value: T, free_id: Option<usize>) -> usize {
        let index = match free_id {
            Some(index) => index,
            None => {
                if self.next < self.size {
                    let index = self.next;
                    self.next += 1;
                    index
                } else {
                    panic!("Out of memory");
                }
            },
        };

        let ptr = unsafe { self.memory.add(index) };

        unsafe { ptr.write(value) };

        index
    }

    pub fn deallocate(&mut self, index: usize) {
        let ptr = unsafe { self.memory.add(index) };

        unsafe { ptr.drop_in_place()}
    }
}

impl<T> Drop for SimplePool<T> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.memory.as_ptr() as *mut u8, self.layout);
        }
    }
}