use std::collections::BTreeSet;
use super::prev_power_of_two;
use alloc::collections::BTreeSet;
use core::cmp::min;
use core::ops::Range;

#[cfg(feature = "use_spin")]
use core::ops::Deref;
#[cfg(feature = "use_spin")]
use spin::Mutex;


pub struct BuddyAllocator{
    //最大支持2^32次方
    link_list:[BTreeSet<usize>;32],

    allocated:usize, //已经分配
    sum :usize,
}

impl BuddyAllocator {
    //创建一个空的伙伴分配器
    pub fn new() -> Self{
        BuddyAllocator { 
            link_list:Default::default(), 
            allocated: 0, 
            sum: 0,
         }
    }
    
    /// 从分配器中分配内存[start,end)
    pub fn alloc_frame(&mut self, start:usize,end:usize){
        assert!(start <= end);

        let mut total = 0;
        let mut current_start = start;

        while current_start < end{
            let lowbit =  if current_start > 0{
                current_start & (!current_start+1)
            }else{
                32
            };

            let size = min(lowbit,prev_power_of_two(end-current_start));
            total += size;

            self.link_list[size.trailing_zeros() as usize].insert(current_start);
            current_start += size;
        }
        self.sum += total;
    }

    pub fn insert(&mut self, range: Range<usize>){
        self.alloc_frame(range.start, range.end);
    }

    pub fn alloc(&mut self, count: usize) -> Option<usize>{
        let size = count.next_power_of_two();
        let class = size.trailing_zeros() as usize;
    
        for i in class..self.link_list.len(){
            //找到第一个不为空的块
            if !self.link_list[i].is_empty(){
                for j in (class+1..i+1).rev(){
                    if let Some(block_ref) = self.link_list[j].iter().next(){
                        let block = *block_ref;
                        self.link_list[j-1].insert(block+(1 << (j-1)));
                        self.link_list[j-1].insert(block);
                        self.link_list[j].remove(&block);
                    }else{
                        return None;
                    }
                }
                let result = self.link_list[class].iter().next().clone();
                if let Some(result_ref) = result{
                    let result = *result_ref;
                    self.link_list[class].remove(&result);
                    self.allocated += size;
                    return  Some(result);
                }else{
                    return None;
                }
            }
        }
        None
    }

    pub fn dealloc(&mut self, frame: usize, count: usize){
       let size = count.next_power_of_two();
       let class = size.trailing_zeros() as usize;
       
       //合并链表中空闲的块
       let mut current_ptr = frame;
       let mut current_class  = class;
       
       while  current_class < self.link_list.len(){
           let buddy =  current_ptr ^ (1 << current_class);
           if self.link_list[current_class].remove(&buddy) == true{
               current_ptr = min(current_ptr,buddy);
               current_class += 1;
           }else{
               self.link_list[current_class].insert(current_ptr);
               break;
           }
       }
       self.allocated -= size;
    }

}

#[cfg(feature = "use_spin")]
pub struct LockedFrameAllocator(Mutex<BuddyAllocator>);

#[cfg(feature = "use_spin")]
impl LockedFrameAllocator {
    /// Creates an empty heap
    pub fn new() -> LockedFrameAllocator {
        LockedFrameAllocator(Mutex::new(BuddyAllocator::new()))
    }
}

#[cfg(feature = "use_spin")]
impl Deref for LockedFrameAllocator {
    type Target = Mutex<BuddyAllocator>;

    fn deref(&self) -> &Mutex<BuddyAllocator> {
        &self.0
    }
}
