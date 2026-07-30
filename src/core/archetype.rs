use std::{cell::UnsafeCell, collections::HashMap};

use crate::core::component::ComponentColumn;

use super::{
    component::{BundleComponent, Component},
    entity::EntityId,
    error::ArchetypeError,
};

/// Storage for every entity that shares the exact same set of component types, laid out as one
/// [`ComponentColumn`] per type. [`EntityManager`](super::entity_manager::EntityManager)
/// migrates an entity between archetypes whenever its component set changes.
pub struct Archetype {
    pub(crate) components: HashMap<std::any::TypeId, Box<dyn ComponentColumn>>,
    pub(crate) entities: Vec<EntityId>,
}

/// An entity's components, detached from their archetype during a migration (e.g. via
/// [`Archetype::migrate_entity_to_other_archetype`]) before being re-inserted elsewhere.
pub type MovedEntity = HashMap<std::any::TypeId, Box<dyn ComponentColumn>>;

impl Archetype {
    /// Creates a new archetype containing a single entity with the given component bundle.
    pub fn new(entity_id: EntityId, components: impl BundleComponent) -> Self {
        let components = components.create_map_components(entity_id);
        Self {
            components,
            entities: vec![entity_id],
        }
    }
    /// Creates a new archetype from an entity's components detached during a migration.
    pub fn new_from_migration(entity_id: EntityId, components: MovedEntity) -> Self {
        Self {
            components,
            entities: vec![entity_id],
        }
    }

    /// Adds an entity with the given component bundle to this (already-matching) archetype.
    pub fn add_entity(&mut self, entity_id: EntityId, components: impl BundleComponent) {
        for (type_id, column) in components.create_map_components(entity_id) {
            match self.components.get_mut(&type_id) {
                Some(existing) => existing.merge_from(column),
                None => {
                    self.components.insert(type_id, column);
                }
            }
        }
        self.entities.push(entity_id);
    }

    /// Re-inserts an entity (and its previously detached components) into this archetype
    /// after a migration.
    pub fn add_entity_migrated(&mut self, entity_id: EntityId, components: MovedEntity) {
        for (type_id, column) in components {
            match self.components.get_mut(&type_id) {
                Some(existing) => existing.merge_from(column),
                None => {
                    self.components.insert(type_id, column);
                }
            }
        }
        self.entities.push(entity_id);
    }

    /// Removes `entity_id` from this archetype and returns its detached components, so the
    /// caller can insert them (possibly alongside new/removed component types) into another
    /// archetype. Errors if the entity isn't in this archetype.
    pub fn migrate_entity_to_other_archetype(
        &mut self,
        entity_id: EntityId,
    ) -> Result<(EntityId, MovedEntity), ArchetypeError> {
        let index = self.entities.iter().position(|&x| x == entity_id);
        match index {
            Some(index) => {
                let mut components = HashMap::new();
                for (type_id, column) in self.components.iter_mut() {
                    components.insert(*type_id, column.extract_single(index));
                }
                self.entities.swap_remove(index);
                Ok((entity_id, components))
            }
            None => Err(ArchetypeError::EntityNotFound),
        }
    }

    /// Removes `entity_id` and all its components from this archetype. Errors if the entity
    /// isn't in this archetype.
    pub fn remove_entity(&mut self, entity_id: EntityId) -> Result<(), ArchetypeError> {
        let index = self.entities.iter().position(|&x| x == entity_id);
        match index {
            Some(index) => {
                for column in self.components.values_mut() {
                    column.swap_remove_drop(index);
                }
                self.entities.swap_remove(index);
                Ok(())
            }
            None => Err(ArchetypeError::EntityNotFound),
        }
    }

    /// Returns whether this archetype includes component type `type_id`.
    pub fn has_type(&self, type_id: std::any::TypeId) -> bool {
        self.components.contains_key(&type_id)
    }

    /// Returns whether this archetype has no entities.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Returns a raw pointer to `entity_id`'s component `T` within this archetype, or `None`
    /// if the entity or component type isn't present.
    pub fn get_component<T: Component + 'static>(&self, entity_id: EntityId) -> Option<*const T> {
        let local_index = self.entities.iter().position(|&id| id == entity_id)?;
        let column = self.components.get(&std::any::TypeId::of::<T>())?;
        let list = column.as_any().downcast_ref::<UnsafeCell<Vec<T>>>()?;
        let value = unsafe { (&*list.get()).get(local_index)? };

        Some(value as *const _)
    }

    /// Mutable counterpart to [`Archetype::get_component`].
    pub fn get_component_mut<T: Component + 'static>(&self, entity_id: EntityId) -> Option<*mut T> {
        let local_index = self.entities.iter().position(|&id| id == entity_id)?;

        let column = self.components.get(&std::any::TypeId::of::<T>())?;
        let list = column.as_any().downcast_ref::<UnsafeCell<Vec<T>>>()?;
        let value = unsafe { (&mut *list.get()).get_mut(local_index)? };

        Some(value as *mut _)
    }
}

#[test]
fn test_archetype() {
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Health(i32);
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Position(i32, i32);
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Velocity(i32, i32);

    let mut arch = Archetype::new(0, (Health(100), Position(0, 0), Velocity(0, 0)));
    arch.add_entity(1, (Health(200), Position(1, 1), Velocity(1, 1)));

    let (entity_id, moved_entity) = arch.migrate_entity_to_other_archetype(0).unwrap();
    assert_eq!(entity_id, 0);
    assert_eq!(moved_entity.len(), 4); // Health + Position + Velocity + Entity
}
