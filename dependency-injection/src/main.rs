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

impl<Func, A1> Callable<(A1,)> for Func
where
    Func: Fn(A1),
{
    fn call(&self, (a1,): (A1,)) {
        (self)(a1,);
    }
}

impl<Func, A1, A2> Callable<(A1, A2)> for Func
where
    Func: Fn(A1, A2),
{
    fn call(&self, (a1, a2): (A1, A2)) {
        (self)(a1, a2);
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

impl<A: FromTypeMap> FromTypeMap for (A,) {
    fn from_type_map(type_map: &TypeMap) -> Self {
        (A::from_type_map(type_map),)
    }
}

impl<A: FromTypeMap, B: FromTypeMap> FromTypeMap for (A, B) {
    fn from_type_map(type_map: &TypeMap) -> Self {
        (A::from_type_map(type_map), B::from_type_map(type_map))
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
    fn injects_multiple_dependencies_based_on_arg_type() {
        let mut container = TypeMap::default();

        struct Point {
            x: u8,
            y: u8,
        }

        container.bind(Data::new(String::from("hello mul injects")));
        container.bind(Data::new(Point { x: 3, y: 5 }));

        container.call(|string: Data<String>, p1: Data<Point>| {
            assert_eq!(string.as_str(), "hello mul injects");
            assert_eq!(p1.x, 3);
        })
    }

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
