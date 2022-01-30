use super::{align_up, Locked};
use  std::alloc::{GlobalAlloc,Layout};
use core::ptr;

struct LinearAllocator{
    heap_start:usize,
    heap_end:usize,
    next:usize,
    allocations: usize,
}

impl LinearAllocator {
    
    //创建一个空的线性分配器
    pub const fn new() -> Self{
        LinearAllocator{
            heap_start:0,
            heap_end:0,
            next:0,
          allocations:0,  
        }
    }

    //使用给定的堆边界初始化线性分配器,该方法为非安全，因为调用者必须保证提供的内存范围未被使用。
// 同时，该方法只能被调用一次。

    pub unsafe fn init(&mut self,heap_start:usize,heap_end:usize){
        self.heap_start = heap_start;
        self.heap_end = heap_start+heap_end;
        self.next = heap_start;
    }

}

unsafe impl GlobalAlloc for Locked<LinearAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        
        let mut bump = self.lock();
        let alloc_start = align_up(bump.next,layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        // 内存空间不足
        if alloc_end > bump.heap_end{
            //可以选择panic掉
            ptr::null_mut()
        }else{
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut bump = self.lock();
        if bump.allocations < 0{
            // TODO 如果内存未使用,进行释放就panic。
            panic!("");
        }
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_next;
        }
    }
}
