use super::{component::Component, world::World};

pub type EntityId = u32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: EntityId,
    pub entity_location: usize,
}

impl Entity {
    pub(crate) fn new(id: EntityId, entity_location: usize) -> Self {
        Entity {
            id,
            entity_location,
        }
    }

    pub fn get_component<T: 'static + Component>(&self, world: &World) -> Option<&T> {
        match world.entity_manager.borrow().get_component::<T>(*self) {
            Ok(component) => Some(unsafe { &*component }),
            Err(e) => panic!("{:?}", e),
        }
    }

    pub fn get_component_mut<T: 'static + Component>(&self, world: &World) -> Option<&mut T> {
        match world.entity_manager.borrow().get_component_mut::<T>(*self) {
            Ok(component) => Some(unsafe { &mut *component }),
            Err(e) => panic!("{:?}", e),
        }
    }
}
