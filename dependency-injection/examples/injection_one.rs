use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

#[derive(Default)]
pub struct TypeMap {
    bindings: HashMap<TypeId, Box<dyn Any>>,
}

impl TypeMap {
    pub fn bind<T: Any>(&mut self, val: T) {
        self.bindings.insert(val.type_id(), Box::new(val));
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.bindings
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    pub fn call<F, Args>(&self, callable: F)
    where
        F: Callable<Args>,
        Args: FromTypeMap,
    {
        callable.call(Args::from_type_map(self));
    }
}

pub trait Callable<Args> {
    fn call(&self, args: Args);
}

impl<Func, Arg1> Callable<Arg1> for Func
where
    Func: Fn(Arg1),
{
    fn call(&self, arg1: Arg1) {
        (self)(arg1);
    }
}

pub trait FromTypeMap {
    fn from_type_map(type_map: &TypeMap) -> Self;
}

impl FromTypeMap for i32 {
    fn from_type_map(type_map: &TypeMap) -> Self {
        *type_map.get::<Self>().expect("type not found")
    }
}

pub struct Data<T: ?Sized>(Arc<T>);

impl<T> Data<T> {
    pub fn new(val: T) -> Self {
        Data(Arc::new(val))
    }
}

impl<T: ?Sized> Data<T> {
    pub fn get(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: ?Sized> Clone for Data<T> {
    fn clone(&self) -> Self {
        Data(self.0.clone())
    }
}

impl<T: ?Sized> Deref for Data<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Arc<T> {
        &self.0
    }
}

impl<T: ?Sized + 'static> FromTypeMap for Data<T> {
    fn from_type_map(type_map: &TypeMap) -> Self {
        type_map.get::<Self>().expect("type not found").clone()
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_type_can_be_bound_and_resolved() {
        let mut container = TypeMap::default();
        container.bind::<i32>(42);
        assert_eq!(container.get::<i32>(), Some(&42));
    }

    #[test]
    fn a_type_can_be_bound_and_resolved_through_inference() {
        let mut container = TypeMap::default();
        container.bind(42);
        assert_eq!(container.get(), Some(&42));
    }

    #[test]
    fn the_vales_method_can_be_accessed_through_deref() {
        let mut container = TypeMap::default();
        container.bind(Data::new(String::from("test hello injection")));
        container.call(|data: Data<String>| {
            assert_eq!(data.as_str(), "test hello injection");
        });
    }

    #[test]
    fn injects_dependency_based_on_argument_type() {
        let mut container = TypeMap::default();
        container.bind(Data::new(42));
        container.call(|data: Data<i32>| {
            assert_eq!(data.get(), &42);
        })
    }
}
