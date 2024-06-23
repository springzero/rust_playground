use std::ptr::{addr_of, addr_of_mut};
use std::mem::MaybeUninit;

struct Bed { cat: MaybeUninit<Cat> }
impl Bed {
    fn new(slot: &mut MaybeUninit<Self>) {
        let this: *mut Self = slot.as_mut_ptr();
        Cat::new(4, unsafe { &mut (*this).cat });
    }
}

struct Cat { age: u8 }
impl Cat {
    fn new(age: u8, slot: &mut MaybeUninit<Self>) {
        let this: *mut Self = slot.as_mut_ptr();
        unsafe { 
            addr_of_mut!((*this).age).write(age);
            dbg!(addr_of!(*this)); // ← second call to `addr_of!`
        };
    }
}

fn main() {
    let mut slot = MaybeUninit::uninit();
    dbg!(addr_of!(slot));      // ← first call to `addr_of!`
    Bed::new(&mut slot);
    let bed: &mut Bed = unsafe { (slot).assume_init_mut() };
    dbg!(addr_of!(*bed));      // ← third call to `addr_of!`
}