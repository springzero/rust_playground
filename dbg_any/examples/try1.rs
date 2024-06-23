use std::ptr::addr_of;

struct Cat { age: u8 }
impl Cat {
    fn new(age: u8) -> Self {
        let this = Self { age };
        dbg!(addr_of!(this)); // ← first call to `addr_of!`
        this
    }
}

fn main() {
    let cat = Cat::new(4);
    dbg!(addr_of!(cat));      // ← second call to `addr_of!`
}