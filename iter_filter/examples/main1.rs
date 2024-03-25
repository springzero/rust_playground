use rand::prelude::*;
use std::ops::Rem;

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let mut it = numbers.iter();

    let needs_filtering = random::<usize>().rem(2) == 0;
    if needs_filtering {
        it = it.filter(|x| x.rem(2) == 0);
    };

    it.for_each(|x| {
        println!("{:?}", x);
    });

    println!("Hello, world!");
}
