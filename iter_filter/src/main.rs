use rand::prelude::*;
use std::ops::Rem;

struct PipelineBuilder<'a, T> {
    iter: Box<dyn Iterator<Item = T> + 'a>,
}

impl<'a, T: 'a> PipelineBuilder<'a, T> {
    fn new(iter: impl Iterator<Item = T> + 'a) -> Self {
        Self {
            iter: Box::new(iter)
        }
    }
}

impl<'a, T: 'a> PipelineBuilder<'a, T> {
    fn with_filter(self, cond: impl FnMut(&T) -> bool + 'a) -> Self {
        Self {
            iter: Box::new(self.iter.filter(cond))
        }
    }
    fn with_map<S>(self, f: impl FnMut(T) -> S + 'a) -> PipelineBuilder<'a, S> {
        PipelineBuilder {
            iter: Box::new(self.iter.map(f))
        }
    }
    fn for_each(self, f: impl FnMut(T) -> () + 'a) {
        self.iter.for_each(f)
    }
}

fn main() {
    let mut it = PipelineBuilder::new((1..10).into_iter());

    let cnt = random::<usize>().rem(10);
    for i in 0..cnt {
        it = it.with_map(move |x| x + i)
    }

    it
        .with_filter(|x| x.rem(2) == 0)
        .for_each(|x| print!("{x} "));
    
    println!("Hello, world!");
}
