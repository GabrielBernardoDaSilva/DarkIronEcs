use std::{any::Any, cell::UnsafeCell, collections::HashMap};

use super::entity::EntityId;

/// Marker trait for any `'static` type usable as a component. Blanket-implemented for every
/// such type, so no manual `impl` is needed.
pub trait Component: Any {}
impl<T: Any> Component for T {}

pub trait ComponentColumn: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn extract_single(&mut self, index: usize) -> Box<dyn ComponentColumn>;

    fn merge_from(&mut self, other: Box<dyn ComponentColumn>);

    fn swap_remove_drop(&mut self, index: usize);
}

impl<T: Component + 'static> ComponentColumn for UnsafeCell<Vec<T>> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn len(&self) -> usize {
        unsafe { (*self.get()).len() }
    }

    fn extract_single(&mut self, index: usize) -> Box<dyn ComponentColumn> {
        let value = unsafe { (*self.get()).swap_remove(index) };
        Box::new(UnsafeCell::new(vec![value]))
    }

    fn merge_from(&mut self, other: Box<dyn ComponentColumn>) {
        let mut other_vec = other
            .into_any()
            .downcast::<UnsafeCell<Vec<T>>>()
            .expect("ComponentColumn::merge_from: expected UnsafeCell<Vec<T>>")
            .into_inner();

        unsafe { (*self.get()).append(&mut other_vec) };
    }

    fn swap_remove_drop(&mut self, index: usize) {
        unsafe {
            (*self.get()).swap_remove(index);
        }
    }
}

/// Implemented for tuples of components (up to 26 elements), letting
/// [`World::create_entity`](super::world::World::create_entity) and friends accept
/// `(Health(100), Position(0, 0))`-style bundles directly.
pub trait BundleComponent {
    fn create_map_components(
        self,
        entity_id: EntityId,
    ) -> HashMap<std::any::TypeId, Box<dyn ComponentColumn>>;
    fn get_types_id(&self) -> Vec<std::any::TypeId>;
}

macro_rules! impl_bundle_component {
    // Base case: Implement for a single element tuple
    ( $head:ident ) => {
        impl< $head: 'static > BundleComponent for ($head,) {
            fn create_map_components(self, entity_id: EntityId) -> HashMap<std::any::TypeId, Box<dyn ComponentColumn>> {
                let mut map = HashMap::new();
                map.insert(
                    std::any::TypeId::of::<$head>(),
                    (Box::new(UnsafeCell::new(vec![self.0])) as Box<dyn ComponentColumn>),
                );
                map.insert(
                    std::any::TypeId::of::<super::entity::Entity>(),
                    Box::new(UnsafeCell::new(vec![super::entity::Entity::new(entity_id, 0)])) as Box<dyn ComponentColumn>,
                );
                map
            }

            fn get_types_id(&self) -> Vec<std::any::TypeId> {
                vec![std::any::TypeId::of::<$head>(), std::any::TypeId::of::<super::entity::Entity>()]
            }
        }


    };
    // Recursive case: Implement for tuples with more than one element
    ( $head:ident, $($tail:ident),+ ) => {
        impl_bundle_component!($($tail),+);
        impl< $head: 'static, $($tail: 'static ),* > BundleComponent for ($head, $($tail),*) {

            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            fn create_map_components(self,  entity_id: EntityId) -> HashMap<std::any::TypeId, Box<dyn ComponentColumn>> {
                let mut map = HashMap::new();
                let ($head, $($tail),*) = self;
                map.insert(
                    std::any::TypeId::of::<$head>(),
                    (Box::new(UnsafeCell::new(vec![$head])) as Box<dyn ComponentColumn>),
                );
                $(
                    map.insert(
                        std::any::TypeId::of::<$tail>(),
                        (Box::new(UnsafeCell::new(vec![$tail])) as Box<dyn ComponentColumn>),
                    );
                )*
                map.insert(
                    std::any::TypeId::of::<super::entity::Entity>(),
                    (Box::new(UnsafeCell::new(vec![super::entity::Entity::new(entity_id, 0)])) as Box<dyn ComponentColumn>),
                );

                map
            }

            fn get_types_id(&self) -> Vec<std::any::TypeId> {
                vec![std::any::TypeId::of::<$head>(),
                $(std::any::TypeId::of::<$tail>()),*,
                std::any::TypeId::of::<super::entity::Entity>()]
            }
        }
    }
} // Generate implementations for tuples up to length 26
impl_bundle_component!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);

#[cfg(test)]
mod component_column_tests {
    use super::*;
    #[test]
    fn component_column_extract_and_merge_roundtrip() {
        let mut boxed: Box<dyn ComponentColumn> = Box::new(UnsafeCell::new(vec![10i32, 20, 30]));

        let extracted = boxed.extract_single(0); // swap_remove(0) -> column becomes [30, 20]

        let remaining = boxed
            .as_any()
            .downcast_ref::<UnsafeCell<Vec<i32>>>()
            .unwrap();
        assert_eq!(unsafe { &*remaining.get() }, &vec![30, 20]);

        let mut target: Box<dyn ComponentColumn> = Box::new(UnsafeCell::new(vec![99i32]));
        target.merge_from(extracted);

        let merged = target
            .as_any()
            .downcast_ref::<UnsafeCell<Vec<i32>>>()
            .unwrap();
        assert_eq!(unsafe { &*merged.get() }, &vec![99, 10]);
    }

    #[test]
    fn component_column_swap_remove_drop_does_not_panic_and_shrinks_len() {
        let mut boxed: Box<dyn ComponentColumn> = Box::new(UnsafeCell::new(vec!["a", "b", "c"]));
        assert_eq!(boxed.len(), 3);
        boxed.swap_remove_drop(1);
        assert_eq!(boxed.len(), 2);
    }
}
