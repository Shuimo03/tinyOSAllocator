use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); //获取可变引用

        let alloc

    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout){

    }
}