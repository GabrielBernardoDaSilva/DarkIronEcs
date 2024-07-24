use std::{cell::RefCell, collections::HashMap};

use super::entity::EntityId;

pub trait Component {}
pub trait BundleComponent {
    fn create_map_components(self, entity_id: EntityId)
        -> HashMap<std::any::TypeId, ComponentList>;
    fn get_types_id(&self) -> Vec<std::any::TypeId>;
}

impl<T> Component for T {}


pub struct ComponentList {
    pub components: Vec<Box<RefCell<dyn Component>>>,
}

impl ComponentList {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn add<T: Component + 'static>(&mut self, component: T) {
        self.components.push(Box::new(RefCell::new(component)));
    }

    pub fn get<T: Component + 'static>(&self, index: usize) -> Option<&T> {
        let component = self.components.get(index);
        match component {
            Some(component) => {
                let ptr = component.as_ref().as_ptr();
                let ptr = ptr as *const T;
                unsafe { Some(&*ptr) }
            }
            None => None,
        }
    }

    pub fn get_mut<T: Component + 'static>(&self, index: usize) -> Option<&mut T> {
        let component = self.components.get(index);
        match component {
            Some(component) => {
                let ptr = component.as_ref().as_ptr();
                let ptr = ptr as *mut T;
                unsafe { Some(&mut *ptr) }
            }
            None => None,
        }
    }

    pub fn remove(&mut self, index: usize) -> Box<RefCell<dyn Component>> {
        self.components.remove(index)
    }
}

macro_rules! impl_bundle_component {
    // Base case: Implement for a single element tuple
    ( $head:ident ) => {
        impl< $head: 'static > BundleComponent for ($head,) {
            fn create_map_components(self, entity_id: EntityId) -> HashMap<std::any::TypeId, ComponentList> {
                let mut map = HashMap::new();
                let mut component_list = ComponentList::new();
                component_list.add(self.0);
                map.insert(std::any::TypeId::of::<$head>(), component_list);

                let mut component_list = ComponentList::new();
                component_list.add(super::entity::Entity::new(entity_id, 0));
                map.insert(std::any::TypeId::of::<super::entity::Entity>(), component_list);
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
            fn create_map_components(self,  entity_id: EntityId) -> HashMap<std::any::TypeId, ComponentList> {
                let mut map = HashMap::new();
                let ($head, $($tail),*) = self;
                let mut component_list = ComponentList::new();
                component_list.add($head);
                map.insert(std::any::TypeId::of::<$head>(), component_list);
                $(
                    let mut component_list = ComponentList::new();
                    component_list.add($tail);
                    map.insert(std::any::TypeId::of::<$tail>(), component_list);
                )*
                map
            }

            fn get_types_id(&self) -> Vec<std::any::TypeId> {
                vec![std::any::TypeId::of::<$head>(), $(std::any::TypeId::of::<$tail>()),*]
            }
        }
    }
}

// Generate implementations for tuples up to length 26
impl_bundle_component!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);

#[test]
fn test() {

    struct Health(i32);
    let mut component_list = ComponentList::new();
    component_list.add(Health(100));
    let health = component_list.get::<Health>(0).unwrap();
    println!("Health: {}", health.0);
    let health = component_list.get_mut::<Health>(0).unwrap();
    health.0 = 50;
    println!("Health: {}", health.0);
}
