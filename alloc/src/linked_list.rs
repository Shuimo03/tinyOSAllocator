/**
 * An intrusive linked list
 * 参考cs140e 2018 winner(https://cs140e.sergio.bz/)
 */

#[allow(dead_code)]
use std::{fmt,ptr};

#[derive(Clone, Copy)]
pub struct LinkedList{
    head: *mut usize,
}

unsafe impl Send for LinkedList {}

impl LinkedList {
    //Returns a new, empty linked list.
    pub const fn new() -> LinkedList{
        LinkedList { head: ptr::null_mut() }
    }

    // returns `true` if the list is empty and `false` otherwise
    pub fn is_empty(&self) -> bool{
        let res = false;

      if self.head.is_null(){
        return res;
      }
      return !res;
    }

     /// Pushes the address `item` to the front of the list.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `item` refers to unique, writeable memory at
    /// least `usize` in size that is valid as long as `item` resides in `self`.
    /// Barring the uniqueness constraint, this is equivalent to ensuring that
    /// `*item = some_usize` is a safe operation as long as the pointer resides
    /// in `self`.
    /// 

    pub unsafe fn push(&mut self, item: *mut usize){
        *item = self.head as usize;
        self.head = item;
    }

     /// Removes and returns the first item in the list, if any.
     pub unsafe fn pop(&mut self) -> Option<*mut usize>{
        let value = self.peek()?;
        self.head = unsafe {
            *value as *mut usize
        };
        Some(value)
     }

    /// Returns the first item in the list without removing it, if any.
    pub unsafe fn peek(&self) -> Option<*mut usize>{
        match self.is_empty() {
            true => None,
            false => Some(self.head),
        }
    }

    /// Returns an iterator over the items in this list.
    pub fn iter(&self) -> Iter {
        Iter {
             current: self.head, 
            _list: self, 
         }
    }
    /// Return an mutable iterator over the items in the list
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            list: self,
        }
    }
}

impl fmt::Debug for LinkedList {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
         f.debug_list().entries(self.iter()).finish()
     }
}

pub struct Iter<'a>{
    _list: &'a LinkedList,
    current: *mut usize,
}

impl <'a> Iterator for Iter<'a>{
    
    fn next(&mut self) -> Option<Self::Item>{
        let mut list = LinkedList{head:self.current};
        let value = list.pop()?;
        self.current = list.head;
        Some(value);
    }
}

/// An item returned from a mutable iterator of a `LinkedList`

pub struct Node{
    prev: *mut usize,
    value: *mut usize
}

impl Node {
    /// Removes and returns the value of this item from the linked list it
    /// belongs to.
    
    pub fn pop(self) -> *mut usize{
        unsafe{
            *(self.prev) = *(self.value);
            self.value
        }
    }

     /// Returns the value of this element.
     pub fn value(&self) -> *mut usize{
         self.value
     }
}

/// An iterator over the items of the linked list allowing mutability.
pub struct IterMut<'a> {
    _list: &'a mut LinkedList,
    prev: *mut usize,
    current: *mut usize
}

impl<'a> Iterator for IterMut<'a> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let mut list = LinkedList { head: self.current };
        let value = list.pop()?;
        let prev = self.prev;
        self.prev = self.current;
        self.current = list.head;
        Some(Node { prev, value })
    }
}