#![cfg_attr(feature = "const_fn", feature(const_mut_refs, const_fn_fn_ptr_basics))]
#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;
extern crate alloc;

#[cfg(feature = "use_spin")]
extern crate spin;

use core::alloc::{GlobalAlloc,Layout};
use core::cmp::{max,min};
use core::fmt;
use core::mem::size_of;
#[cfg(feature = "use_spin")]
use core::ops::Deref;
use core::ptr::NonNull;
#[cfg(feature="use_spin")]
use spin::Mutex;

#[cfg(test)]
mod test;
mod buddy;

pub use buddy::*;

pub struct Heap<const ORDER: usize>{
    free_list:[linked_list::LinkedList; ORDER],

    user:usize,
    allocated:usize, //已经分配
    sum :usize,
}

impl <const ORDER: usize> Heap<ORDER> {
    
    //初始化heap内存
    pub const  fn new()-> Self{
        Heap { 

            free_list:[linked_list::LinkedList::new();ORDER],
            user: 0, 
            allocated: 0,
            sum: 0,
        }
    }

    //创建一块空的heap内存
    pub const fn empty() -> Self{
        Self::new()
    }

    //从内存中分配heap,分配的范围在[start,end)
    pub unsafe fn free_heap(&mut self, mut start: usize,mut end:usize){
        //避免在某些平台上访问内存对齐
        start = (start+size_of::<usize>()-1) & (!size_of::<usize>()+1);
        end = end & (!size_of::<usize>()+1);
        assert!(start<= end);
        let mut sum = 0;
        let mut current_start = start;

        while current_start+size_of::<usize>() <= end{
            let lowbit = current_start & (!current_start+1);
            let size = min(lowbit,prev_power_of_two(end-current_start));
            sum += size;

            self.free_list[size.trailing_zeros() as usize].push(current_start as *mut usize);
            current_start += size;
        }
        self.sum += sum;
    }

    pub unsafe fn init(&mut self,start: usize,size: usize){
        self.free_heap(start, start+size);
    }

    
    pub fn alloc(&mut self,layout:Layout) -> Result<NonNull<u8>,()>{
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );

        let class = size.trailing_zeros() as usize;
        for i in class..self.free_list.len(){
            //找到第一个不为空的块
            if !self.free_list[i].is_empty(){
                // Split buffers
                for j in (class+1..i+1).rev(){
                    if let Some(block) = self.free_list[j].pop(){
                        unsafe{
                            self.free_list[j-1].push((block as usize + (1 << (j-1))) as *mut usize);
                            self.free_list[j-1].push(block);
                        }
                    }else{
                        return Err(());
                    }
                }

                let res = NonNull::new(
                    self.free_list[class]
                    .pop().expect("current block should have free space now") as *mut u8,
                );
                if let Some(res) = res{
                    self.user += layout.size();
                    self.allocated += size;
                    return Ok(res);
                }else{
                    return Err(());
                }
            }
        }
        Err(())
    }

    // 从堆上回收内存
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout:Layout){
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let class = size.trailing_zeros() as usize;

        unsafe{
            //回收块到链表中
            self.free_list[class].push(ptr.as_ptr() as *mut usize);
            //合并伙伴块
            let mut current_ptr = ptr.as_ptr() as usize;
            let mut current_class = class;
            while current_class < self.free_list.len(){
                let buddy = current_ptr ^ (1 << current_class);
                let mut flag = false;
                for block in self.free_list[current_class].iter_mut(){
                    if block.value() as usize == buddy{
                        block.pop();
                        flag = true;
                        break;
                    }
                }

                //Free buddy found
                if flag{
                    self.free_list[current_class].pop();
                    current_ptr = min(current_ptr,buddy);
                    current_class += 1;
                    self.free_list[current_class].push(current_ptr as *mut usize);
                }else{
                    break;
                }
            }
        }
        self.user -= layout.size();
        self.allocated -= size;
    }

    pub fn stats_alloc_user(&self) -> usize{
        self.user
    }

    pub fn stats_alloc_actual(&self) -> usize{
        self.allocated
    }
}

impl <const ORDER: usize> fmt::Debug for Heap<ORDER> {
    fn fmt(&self,fmt:&mut fmt::Formatter) -> fmt::Result{
        fmt.debug_struct("Heap")
            .field("user", &self.user)
            .field("allocated", &self.allocated)
            .field("total", &self.sum)
            .finish()
    }
}

#[cfg(feature = "use_spin")]
pub struct LockedHeap<const ORDER: usize>(Mutex<Heap<ORDER>>);

#[cfg(feature = "use_sin")]
impl <const ORDER: usize> LockedHeap {
    pub const fn new() -> Self{
        LockedHeap(Mutex::new(Heap::<ORDER>::new()))
    }

    pub const fn empty() -> Self{
        LockedHeap(Mutex::new(Heap::<ORDER>::new()))
    }
}

#[cfg(feature = "use_spin")]
impl <const ORDER:usize> Deref for LockedHeap<ORDER> {
    type Target = Mutex<Heap<ORDER>>;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

#[cfg(feature="use_spin")]
unsafe impl <const ORDER: usize> GlobalAlloc for LockedHeap<ORDER>{

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout){
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}


#[cfg(feature = "use_spin")]
pub struct LockedHeapWithRescue<const ORDER: usize>{
    inner: Mutex<Heap<ORDER>>,
    rescue: fn(&mut Heap<ORDER>, &Layout),
}

#[cfg(feature = "use_spin")]
impl <const ORDER: usize> LockedHeapWithRescue<ORDER> {
    #[cfg(feature="const_fn")]
    pub const fn new(rescue: fn(&mut Heap<ORDER>, &Layout)) -> Self{
        LockedHeapWithRescue {
            inner: Mutex::new(Heap::<ORDER>::new()),
            rescue,
        }
    }

    #[cfg(not(feature = "const_fn"))]
    pub fn new(rescue: fn(&mut Heap<ORDER>, &Layout))->Self{
        LockedHeapWithRescue{
            inner:Mutex::new(Heap::<ORDER>::new()),
            rescue,
        }
    }
}

#[cfg(feature="use_spin")]
impl <const ORDER: usize> Deref for LockedHeapWithRescue<ORDER>{
    type  Target = Mutex<Heap<ORDER>>;

    fn deref(&self) -> &Self::Target{
        &self.inner
    }
}

#[cfg(feature="use_spin")]
unsafe impl <const ORDER:usize> GlobalAlloc for LockedHeapWithRescue<ORDER> { 
    unsafe fn alloc(&self,layout:Layout) -> *mut u8{
        let mut inner = self.inner.lock();
        match inner.alloc(layout){
            Ok(allocation) => allocation.as_ptr(),
            Err(_) => {
                (self.rescue)(&mut inner, &layout);
                inner
                    .alloc(layout)
                    .ok()
                    .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout:Layout){
        self.inner
        .lock()
        .dealloc(NonNull::new_unchecked(ptr), layout)
    }
}


pub(crate) fn prev_power_of_two(num: usize) ->usize{
    1 << (8 * (size_of::<usize>()) - num.leading_zeros() as usize - 1)
}