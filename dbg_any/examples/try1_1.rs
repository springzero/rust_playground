use std::ptr::addr_of;

struct Bed { cat: Cat }
impl Bed {
    fn new() -> Self {
        let cat = Cat::new(4);
        Self { cat }
    }
}

struct Cat { age: u8 }
impl Cat {
    fn new(age: u8) -> Self {
        let this = Self { age };
        dbg!(addr_of!(this)); // ← first call to `addr_of!`
        this
    }
}

fn main() {
    let bed = Bed::new();
    dbg!(addr_of!(bed));      // ← second call to `addr_of!`
    dbg!(addr_of!(bed.cat));  // ← third call to `addr_of!`
}