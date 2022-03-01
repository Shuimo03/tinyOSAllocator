#![cfg_attr(feature = "const_mut_refs", feature(const_mut_refs))]
#![cfg_attr(
    feature = "alloc_ref",
    feature(allocator_api, alloc_layout_extra, nonnull_slice_from_raw_parts)
)]
#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(feature = "use_spin")]
extern crate spinning_top;

#[cfg(feature = "use_spin")]
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
#[cfg(feature = "alloc_ref")]
use core::alloc::{AllocError, Allocator};
use core::mem::MaybeUninit;
#[cfg(feature = "use_spin")]
use core::ops::Deref;
use core::ptr::NonNull;
#[cfg(test)]

#[cfg(feature = "use_spin")]
use spinning_top::Spinlock;

pub mod linked_list;

pub struct  Heap{
    
}

/// 以2为次方,向上取整,来保证地址对齐
pub fn align_up(addr: usize,aligin:usize) -> usize{
    (addr+aligin-1) & !(aligin-1)
}