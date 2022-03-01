use core::alloc::Layout;
use core::mem;
use core::mem::{align_of,size_of};
use core::ptr::NonNull;
use super::align_up;

pub struct HoleList{
    first: Hole, 
}

impl HoleList {
    // 创建一个空的链表
    #[cfg(not(feature = "const_mut_refs"))]
    pub fn empty()->HoleList{
        HoleList{
            first:Hole{
                size:0,
                next:None,
            }
        }
    }

    #[cfg(feature = "const_mut_refs")]
    pub const fn empty() -> HoleList{
        HoleList{
            first:Hole{
                size:0,
                next:None,
            }
        }
    }

}

/// A block containing free memory. It points to the next hole and thus forms a linked list.
#[cfg(not(test))]
struct Hole{
    size: usize,
    next: Option<&'static mut Hole>,
}
#[cfg(test)]
pub struct Hole{
    pub size:usize,
    pub  next: Option<&'static mut Hole>,
}

impl Hole {
    /// 返回关于Hole的基本信息
    fn info(&self)->HoleInfo{
        HoleInfo { 
            addr: self as *const _ as usize,
            size: self.size,
        }
    }
}

/// hole基本信息
#[derive(Debug,Clone,Copy)]
struct HoleInfo{
    addr: usize,
    size: usize,
}
