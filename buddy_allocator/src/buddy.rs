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
        BuddyAllocator { link_list:Default::default(), 
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

            let size = min(lowbit,);
        }
    }
}
