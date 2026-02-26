use std::{any::Any, cell::UnsafeCell, collections::HashMap};

use super::entity::EntityId;

pub trait Component: Any {}
impl<T: Any> Component for T {}
pub trait BundleComponent {
    fn create_map_components(self, entity_id: EntityId)
        -> HashMap<std::any::TypeId, ComponentList>;
    fn get_types_id(&self) -> Vec<std::any::TypeId>;
}

pub struct ComponentList {
    pub components: Vec<Box<UnsafeCell<dyn Component>>>,
}

impl ComponentList {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn add<T: Component + 'static>(&mut self, component: T) {
        self.components.push(Box::new(UnsafeCell::new(component)));
    }

    pub fn get<T: Component + 'static>(&self, index: usize) -> Option<*const T> {
        let component = self.components.get(index)?;
        let any_ref: &dyn Any = unsafe { &*component.get() };
        any_ref.downcast_ref::<T>().map(|comp| comp as *const T)
    }

    pub fn get_mut<T: Component + 'static>(&self, index: usize) -> Option<*mut T> {
        let component = self.components.get(index)?;
        let any_ref: &mut dyn Any = unsafe { &mut *component.get() };
        any_ref.downcast_mut::<T>().map(|comp| comp as *mut T)
    }

    pub fn remove(&mut self, index: usize) -> Box<UnsafeCell<dyn Component>> {
        self.components.remove(index)
    }
}

impl Default for ComponentList {
    fn default() -> Self {
        Self::new()
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

                let mut component_list = ComponentList::new();
                component_list.add(super::entity::Entity::new(entity_id, 0));
                map.insert(std::any::TypeId::of::<super::entity::Entity>(), component_list);

                map
            }

            fn get_types_id(&self) -> Vec<std::any::TypeId> {
                vec![std::any::TypeId::of::<$head>(),
                $(std::any::TypeId::of::<$tail>()),*,
                std::any::TypeId::of::<super::entity::Entity>()]
            }
        }
    }
}

// Generate implementations for tuples up to length 26
impl_bundle_component!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
