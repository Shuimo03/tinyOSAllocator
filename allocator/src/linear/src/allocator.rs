use alloc::alloc::{GlobalAlloc,Layout};
pub mod bump;

pub struct BumpAllocator{
    heap_head: usize,
    heap_end: usize,
    next: usize,
    cnt: usize, // 统计已分配的内存
}

impl BumpAllocator {
    
    pub const fn new() -> Self{
        BumpAllocator{
            heap_head:0,
            heap_end:0,
            next:0,
            cnt:0,
        }
    }

    pub unsafe fn init(&mut  self,heap_head:usize,heap_size: usize){
        self.heap_head = heap_head; // 上限
        self.heap_end = heap_head+heap_size; // 下限
        self.next = heap_head;
    }
}


unsafe impl GlobalAlloc for BumpAllocator {

    unsafe fn alloc(&self, layout: Layout) -> *mut u8{
        let alloc_start = self.next;
        self.next = alloc_start+layout.size();
        self.cnt += 1;
        
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout){

    }
}
