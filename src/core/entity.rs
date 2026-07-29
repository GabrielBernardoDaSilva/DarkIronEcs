use super::{component::Component, world::World};

/// Numeric identifier backing an [`Entity`], unique for as long as the entity is alive.
pub type EntityId = u32;

/// A lightweight, `Copy` handle to a spawned entity. Holds no data itself — use
/// [`Entity::get_component`]/[`Entity::get_component_mut`] (or a [`Query`](super::query::Query))
/// against a [`World`] to read its components.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: EntityId,
    /// Index into [`EntityManager::archetypes`](super::entity_manager::EntityManager::archetypes)
    /// of the archetype currently holding this entity's components. Changes whenever the
    /// entity's component set changes.
    pub entity_location: usize,
}

impl Entity {
    pub(crate) fn new(id: EntityId, entity_location: usize) -> Self {
        Entity {
            id,
            entity_location,
        }
    }

    /// Returns a reference to this entity's component `T`, or `None` if the entity doesn't
    /// have one.
    pub fn get_component<T: 'static + Component>(&self, world: &World) -> Option<&T> {
        match world.entity_manager.borrow().get_component::<T>(*self) {
            Ok(component) => Some(unsafe { &*component }),
            Err(_) => None,
        }
    }

    /// Mutable counterpart to [`Entity::get_component`].
    pub fn get_component_mut<T: 'static + Component>(&mut self, world: &World) -> Option<&mut T> {
        match world.entity_manager.borrow().get_component_mut::<T>(*self) {
            Ok(component) => Some(unsafe { &mut *component }),
            Err(_) => None,
        }
    }
}
