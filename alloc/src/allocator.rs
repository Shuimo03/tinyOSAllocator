use spin;
pub mod bump;
pub mod linked_list;


pub struct Locked<A>{
    inner: spin::Mutex<A>,
}

impl <A> Locked<A> {
    pub const fn new(inner:A) -> Self{
        Locked{
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A>{
        self.inner.lock()
    }
}

//内存对齐，要求`align`是2的幂
fn align_up(addr: usize,align: usize) -> usize{
    (addr+align-1) & !(align-1)
}
