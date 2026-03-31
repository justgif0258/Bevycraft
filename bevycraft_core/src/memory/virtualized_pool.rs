use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::transmute;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicUsize;

use bevy::ecs::resource::Resource;
use crossbeam_queue::ArrayQueue;

#[derive(Resource)]
#[rustfmt::skip]
pub struct VirtualizedPool<T> {
    handle      : *mut (),
    paging      : Paging,
    capacity    : Capacity,
    next        : AtomicUsize,
    free_list   : ArrayQueue<*mut ()>,
    _marker     : PhantomData<T>,
}

unsafe impl<T: Sized> Send for VirtualizedPool<T> {}
unsafe impl<T: Sized> Sync for VirtualizedPool<T> {}

impl<T> VirtualizedPool<T> {
    #[inline]
    pub fn new(paging: usize, capacity: usize) -> Option<Self> {
        let alloc_size = paging * capacity;

        let paging = Paging::new(paging)?;
        let capacity = Capacity::new(capacity)?;

        let handle = unsafe {
            #[cfg(target_os = "windows")]
            {
                use winapi::{
                    shared::ntdef::NULL,
                    um::{
                        memoryapi::*,
                        winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE},
                    },
                };

                let ptr = VirtualAlloc(
                    std::ptr::null_mut(),
                    alloc_size,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_READWRITE,
                );

                if ptr == NULL {
                    return None;
                }

                ptr as *mut ()
            }

            #[cfg(unix)]
            {
                use libc::*;

                mmap(
                    std::ptr::null_mut(),
                    alloc_size,
                    PROT_READ | PROT_WRITE,
                    MAP_PRIVATE | MAP_ANONYMOUS,
                    -1,
                    0,
                ) as *mut ()
            }
        };

        Some(Self {
            handle,
            paging,
            capacity,
            next: AtomicUsize::new(0),
            free_list: ArrayQueue::new(capacity.get()),
            _marker: PhantomData,
        })
    }

    #[inline]
    #[must_use]
    pub fn commit(&'_ self) -> Option<Page<'_, T>> {
        let page = unsafe {
            if let Some(ptr) = self.read_ptr() {
                ptr
            } else {
                let next = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                if next >= self.capacity() {
                    self.next.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

                    return None;
                }

                self.handle.add(next * self.paging())
            }
        };

        Some(Page {
            page,
            handle: &self,
            _marker: PhantomData,
        })
    }

    #[inline]
    #[must_use]
    unsafe fn read_ptr(&self) -> Option<*mut ()> {
        self.free_list.pop()
    }

    #[inline]
    unsafe fn write_ptr(&self, ptr: *mut ()) {
        self.free_list.push(ptr).expect("Free list overflowed")
    }

    #[inline]
    #[must_use]
    pub const fn paging(&self) -> usize {
        unsafe { transmute(self.paging) }
    }

    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        unsafe { transmute(self.capacity) }
    }
}

impl<T> Drop for VirtualizedPool<T> {
    fn drop(&mut self) {
        let alloc_size = self.capacity() * self.paging();

        unsafe {
            #[cfg(target_os = "windows")]
            {
                use winapi::um::{memoryapi::*, winnt::MEM_RELEASE};

                VirtualFree(self.handle as *mut _, alloc_size, MEM_RELEASE);
            }

            #[cfg(unix)]
            {
                use libc::*;

                munmap(self.head as *mut _, alloc_size);
            }
        }
    }
}

#[rustfmt::skip]
pub struct Page<'pool, T> {
    page    : *mut (),
    handle  : &'pool VirtualizedPool<T>,
    _marker : PhantomData<&'pool T>,
}

impl<T> Page<'_, T> {
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn read(&self, index: usize) -> &T {
        assert!(self.is_within_bounds(index));

        unsafe { &*((self.page as *mut T).add(index)) }
    }

    #[inline]
    #[track_caller]
    pub fn write(&self, index: usize, value: T) {
        assert!(self.is_within_bounds(index));

        unsafe { (self.page as *mut T).add(index).write(value) };
    }

    #[inline]
    const fn is_within_bounds(&self, index: usize) -> bool {
        index.checked_mul(Self::block_size()).unwrap() < self.handle.paging()
    }

    #[inline]
    const fn block_size() -> usize {
        size_of::<T>()
    }
}

impl<T> Drop for Page<'_, T> {
    fn drop(&mut self) {
        unsafe { self.handle.write_ptr(self.page) }
    }
}

pub type Capacity = NonZeroUsize;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Paging(NonZeroUsize);

impl Default for Paging {
    #[inline]
    fn default() -> Self {
        unsafe { transmute(Self::sys_page_size()) }
    }
}

impl Debug for Paging {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Paging").field(&self.0.get()).finish()
    }
}

impl Paging {
    #[cfg(target_os = "windows")]
    const ALIGNMENT: usize = 4096;

    #[cfg(unix)]
    const ALIGNMENT: usize = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };

    #[inline]
    #[must_use]
    pub const fn new(n: usize) -> Option<Self> {
        if n % Self::ALIGNMENT == 0 {
            unsafe { return transmute(n) }
        }

        None
    }

    #[inline]
    #[must_use]
    pub const unsafe fn new_unchecked(n: usize) -> Self {
        unsafe { Paging(NonZeroUsize::new_unchecked(n)) }
    }

    #[inline]
    #[must_use]
    pub const fn get(&self) -> usize {
        self.0.get()
    }

    #[inline]
    #[must_use]
    pub const fn inner(&self) -> NonZeroUsize {
        self.0
    }

    #[cfg(target_os = "windows")]
    #[inline(always)]
    #[must_use]
    pub fn sys_page_size() -> usize {
        unsafe {
            use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};

            let mut info: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut info);
            info.dwPageSize as usize
        }
    }

    #[cfg(unix)]
    #[inline(always)]
    #[must_use]
    pub fn sys_page_size() -> usize {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }
}
