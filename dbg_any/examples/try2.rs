use std::ptr::{addr_of, addr_of_mut};
use std::mem::MaybeUninit;

struct Cat { age: u8 }
impl Cat {
    fn new(age: u8, slot: &mut MaybeUninit<Self>) {
        let this: *mut Self = slot.as_mut_ptr();
        unsafe { 
           addr_of_mut!((*this).age).write(age);
           dbg!(addr_of!(*this));   // ← second call to `addr_of!`
        };
    }
}

fn main() {
    let mut slot = MaybeUninit::uninit();
    dbg!(addr_of!(slot));      // ← first call to `addr_of!`
    Cat::new(4, &mut slot);
    let cat: &mut Cat = unsafe { (slot).assume_init_mut() };
    dbg!(addr_of!(*cat));      // ← third call to `addr_of!`
}