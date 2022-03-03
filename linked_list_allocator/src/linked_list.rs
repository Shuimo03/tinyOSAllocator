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

    pub unsafe fn new(list_addr: usize,list_size: usize) -> HoleList{
        assert_eq!(size_of::<Hole>(),Self::min_size());

        let aligned_hole_addr = align_up(list_addr,align_of::<Hole>());
        let ptr = aligned_hole_addr as *mut Hole;
        ptr.write(Hole{
            size:list_size.saturating_sub(aligned_hole_addr-list_addr),
            next:None,
        });

        HoleList { 
            first:Hole{
                size:0,
                next:Some(&mut *ptr),
            },
         }
    }

    pub fn align_layout(layout:Layout) -> Layout{
        let mut size = layout.size();
        if size < Self::min_size(){
            size = Self::min_size();
        }
        let size = align_up(size,mem::align_of::<Hole>());
        let layout = Layout::from_size_align(size, layout.align()).unwrap();
        
        layout
    }

    /// 在链表中找到合适的块,如果可以一个块可以容纳layout.size()分配大小的字节，并且拥有layout.align()就表示足够大。
    /// 如果在链表找到这样的块，就会从从中分配一个符合大小内存出去。
    /// 首次适应算法,时间复杂度是O(n)
    pub fn alloc_first_fit(&mut self,layout:Layout) ->Result<(NonNull<u8>, Layout), ()>{
        let aligned_layout = Self::align_layout(layout);

        allocate_first_fit(&mut self.first, aligned_layout).map(|holeinfo| {
            (
                NonNull::new(holeinfo.addr as *mut u8).unwrap(),
                aligned_layout,
            )
        })
    }

    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>,layout:Layout) -> Layout{
        let aligned_layout = Self::align_layout(layout);
        deallocate(
            &mut self.first,
            ptr.as_ptr() as usize,
            aligned_layout.size(),
        );
        aligned_layout
    }

    // 返回最小分配尺寸,用于分配或者回收
    pub fn min_size() -> usize{
        size_of::<usize>() *2
    }

    #[cfg(test)]
    pub fn first_hole(&self) -> Option<(usize,usize)>{
        self.first
        .next
        .as_ref()
        .map(|hole| ((*hole) as *const Hole as usize, hole.size))
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

struct Allocation{
    info:HoleInfo,
    front_padding: Option<HoleInfo>,
    back_padding: Option<HoleInfo>,
}

fn split_hole(hole: HoleInfo, required_layout: Layout) -> Option<Allocation> {
    let required_size = required_layout.size();
    let required_align = required_layout.align();

    let (aligned_addr, front_padding) = if hole.addr == align_up(hole.addr, required_align) {
        // hole has already the required alignment
        (hole.addr, None)
    } else {
        // the required alignment causes some padding before the allocation
        let aligned_addr = align_up(hole.addr + HoleList::min_size(), required_align);
        (
            aligned_addr,
            Some(HoleInfo {
                addr: hole.addr,
                size: aligned_addr - hole.addr,
            }),
        )
    };

    let aligned_hole = {
        if aligned_addr + required_size > hole.addr + hole.size {
            // hole is too small
            return None;
        }
        HoleInfo {
            addr: aligned_addr,
            size: hole.size - (aligned_addr - hole.addr),
        }
    };

    let back_padding = if aligned_hole.size == required_size {
        // the aligned hole has exactly the size that's needed, no padding accrues
        None
    } else if aligned_hole.size - required_size < HoleList::min_size() {
        // we can't use this hole since its remains would form a new, too small hole
        return None;
    } else {
        // the hole is bigger than necessary, so there is some padding behind the allocation
        Some(HoleInfo {
            addr: aligned_hole.addr + required_size,
            size: aligned_hole.size - required_size,
        })
    };

    Some(Allocation {
        info: HoleInfo {
            addr: aligned_hole.addr,
            size: required_size,
        },
        front_padding: front_padding,
        back_padding: back_padding,
    })
}

fn allocate_first_fit(mut previous: &mut Hole, layout: Layout) -> Result<HoleInfo,()>{
        loop{
             let allocation: Option<Allocation> = previous
             .next
             .as_mut()
             .and_then(|current| split_hole(current.info(), layout.clone()));
            
             match allocation {
                 Some(allocation) => {
                     previous.next = previous.next.as_mut().unwrap().next.take();
                    if let Some(padding) = allocation.front_padding{
                         let ptr = padding.addr as *mut Hole;
                         unsafe{
                             ptr.write(Hole{
                                 size:padding.size,
                                 next:previous.next.take(),
                             })
                         }
                         previous.next = Some(unsafe{&mut *ptr});
                         previous = move_helper(previous).next.as_mut().unwrap();
                     }
                     if let Some(padding) = allocation.back_padding{
                            let ptr = padding.addr as *mut Hole;
                            unsafe{
                                ptr.write(Hole{
                                    size: padding.size,
                                    next:previous.next.take(),
                                })
                            }
                            previous.next = Some(unsafe {
                                &mut *ptr
                            });
                     }
                     return Ok(allocation.info);
                 }
                 None if previous.next.is_some() => {
                     previous = move_helper(previous).next.as_mut().unwrap();
                 }
                 None => {
                     return Err(());
                 }
             }
        }
}

fn deallocate(mut hole:&mut Hole, addr: usize, mut size: usize){
     loop {
         assert!(size >= HoleList::min_size());

         let hole_addr = if hole.size == 0{
             0
         }else{
             hole as *mut _ as usize
         };

         assert!(
            hole_addr + hole.size <= addr,
            "invalid deallocation (probably a double free)"
         );

         let next_hole_info = hole.next.as_ref().map(|next|next.info());

         match next_hole_info {
             Some(next) if hole_addr + hole.size == addr && addr + size == next.addr => {

                hole.size += size+next.size;
                hole.next = hole.next.as_mut().unwrap().next.take();
             }
             _ if hole_addr + hole.size == addr => {
                // block is right behind this hole but there is used memory after it
                // before:  ___XXX______YYYYY____    where X is this hole and Y the next hole
                // after:   ___XXXFFFF__YYYYY____    where F is the freed block

                // or: block is right behind this hole and this is the last hole
                // before:  ___XXX_______________    where X is this hole and Y the next hole
                // after:   ___XXXFFFF___________    where F is the freed block

                hole.size += size;
             }
             Some(next) if addr + size == next.addr => {
                // block is right before the next hole but there is used memory before it
                // before:  ___XXX______YYYYY____    where X is this hole and Y the next hole
                // after:   ___XXX__FFFFYYYYY____    where F is the freed block

                hole.next = hole.next.as_mut().unwrap().next.take();
                size += next.size;
                continue;
             }
             Some(next) if next.addr <= addr => {
                 // block is behind the next hole, so we delegate it to the next hole
                // before:  ___XXX__YYYYY________    where X is this hole and Y the next hole
                // after:   ___XXX__YYYYY__FFFF__    where F is the freed block

                hole = move_helper(hole).next.as_mut().unwrap();
                continue;
             }

             _ =>{

                // block is between this and the next hole
                // before:  ___XXX________YYYYY_    where X is this hole and Y the next hole
                // after:   ___XXX__FFFF__YYYYY_    where F is the freed block

                // or: this is the last hole
                // before:  ___XXX_________    where X is this hole
                // after:   ___XXX__FFFF___    where F is the freed block

                let new_hole = Hole{
                    size:size,
                    next:hole.next.take(), // the reference to the Y block (if it exists)
                };

                debug_assert_eq!(addr % align_of::<Hole>(),0);
                let ptr = addr as *mut Hole;
                unsafe{
                    ptr.write(new_hole);
                }
                hole.next = Some(unsafe {
                    &mut *ptr
                });
             }
         }
         break;
     }
}

/// Identity function to ease moving of references.
///
/// By default, references are reborrowed instead of moved (equivalent to `&mut *reference`). This
/// function forces a move.
///
/// for more information, see section “id Forces References To Move” in:
/// https://bluss.github.io/rust/fun/2015/10/11/stuff-the-identity-function-does/
fn move_helper<T>(x:T) -> T{
    x
}