use std::{
    any::TypeId,
    cell::{RefCell, UnsafeCell},
    collections::HashMap,
};

use crate::core::query::QuerySignature;

use super::{
    access::AccessKey,
    archetype::{Archetype, MovedEntity},
    component::{BundleComponent, Component},
    entity::Entity,
    error::QueryError,
    system::SystemParam,
};

/// Owns every entity and its components, grouped into [`Archetype`]s by component-type
/// signature. Most callers interact with it indirectly through [`World`](super::world::World)
/// rather than directly.
pub struct EntityManager {
    pub entities: Vec<Entity>,
    pub archetypes: Vec<Archetype>,
    next_entity_id: u32, // L6: Monotomic Incrementing Counter
    pub(crate) archetype_version: u64,
    pub(crate) query_cache: RefCell<HashMap<QuerySignature, (u64, Vec<usize>)>>,
}

impl SystemParam for &EntityManager {
    fn get_param(
        coordinator: std::rc::Rc<std::cell::RefCell<super::coordinator::Coordinator>>,
    ) -> Self {
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Manager(TypeId::of::<EntityManager>()),
            false,
            "EntityManager",
        );
        unsafe { &*coordinator.borrow().get_entity_manager_mut() }
    }
}

impl SystemParam for &mut EntityManager {
    fn get_param(
        coordinator: std::rc::Rc<std::cell::RefCell<super::coordinator::Coordinator>>,
    ) -> Self {
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Manager(TypeId::of::<EntityManager>()),
            true,
            "EntityManager",
        );
        unsafe { &mut *coordinator.borrow().get_entity_manager_mut() }
    }
}

impl EntityManager {
    /// Creates an empty manager with no entities.
    pub fn new() -> Self {
        EntityManager {
            entities: Vec::new(),
            archetypes: Vec::new(),
            next_entity_id: 0,
            archetype_version: 0,
            query_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Spawns a new entity with the given bundle of components, placing it in the matching
    /// archetype (creating one if none matches yet), and returns its [`Entity`] id.
    pub fn create_entity(&mut self, components: impl BundleComponent) -> Entity {
        let mut types_ids = components.get_types_id();
        let mut entity = Entity::new(self.next_entity_id, 0);
        self.next_entity_id += 1;

        types_ids.sort();

        let archetype_index_opt = self.archetypes.iter().position(|archetype| {
            let mut arch_types_ids = archetype.components.keys().copied().collect::<Vec<_>>();
            arch_types_ids.sort();
            arch_types_ids.iter().eq(types_ids.iter())
        });

        match archetype_index_opt {
            Some(archetype_index) => {
                self.archetypes[archetype_index].add_entity(entity.id, components);
                entity.entity_location = archetype_index;
            }
            None => {
                let archetype = Archetype::new(entity.id, components);
                self.archetypes.push(archetype);
                self.archetype_version += 1;
                entity.entity_location = self.archetypes.len() - 1;
            }
        }

        self.entities.push(entity);
        entity
    }

    /// Removes component `T` from `entity`, migrating it to the matching archetype (or
    /// removing the entity entirely if it has no components left). No-op if `entity` doesn't
    /// exist.
    pub fn remove_component<T: 'static + Component>(&mut self, entity: Entity) {
        let entity_id = entity.id;
        let location = match self.entities.iter().find(|e| e.id == entity_id) {
            Some(e) => e.entity_location,
            None => return,
        };

        let type_id = std::any::TypeId::of::<T>();
        let mut entity_with_components = self.archetypes[location]
            .migrate_entity_to_other_archetype(entity_id)
            .unwrap();
        entity_with_components.1.remove(&type_id);

        let archetype_empty = self.archetypes[location].is_empty();
        if archetype_empty {
            self.remove_archetype(location);
        }

        if entity_with_components.1.is_empty() {
            if let Some(pos) = self.entities.iter().position(|e| e.id == entity_id) {
                self.entities.remove(pos);
            }
        } else {
            // Recria entity com location atualizado após possível remove_archetype
            let updated = match self.entities.iter().find(|e| e.id == entity_id) {
                Some(e) => *e,
                None => return,
            };
            self.move_entity_to_other_archetype(updated, entity_with_components.1);
        }
    }

    /// Adds (or replaces) component `T` on `entity`, migrating it to the matching archetype.
    /// No-op if `entity` doesn't exist.
    pub fn add_component_to_entity<T: 'static + Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) {
        let entity_id = entity.id;
        let location = match self.entities.iter().find(|e| e.id == entity_id) {
            Some(e) => e.entity_location,
            None => return,
        };

        let type_id = std::any::TypeId::of::<T>();
        let mut entity_with_components = self.archetypes[location]
            .migrate_entity_to_other_archetype(entity_id)
            .unwrap();
        entity_with_components.1.insert(
            type_id,
            Box::new(UnsafeCell::new(vec![component]))
                as Box<dyn super::component::ComponentColumn>,
        );

        let archetype_empty = self.archetypes[location].is_empty();
        if archetype_empty {
            self.remove_archetype(location);
        }

        let updated = match self.entities.iter().find(|e| e.id == entity_id) {
            Some(e) => *e,
            None => return,
        };
        self.move_entity_to_other_archetype(updated, entity_with_components.1);
    }

    /// Removes `entity` and all of its components. No-op if `entity` doesn't exist.
    pub fn remove_entity(&mut self, entity: Entity) {
        let entity_id = entity.id;
        let location = match self.entities.iter().find(|e| e.id == entity_id) {
            Some(e) => e.entity_location,
            None => return,
        };

        self.archetypes[location].remove_entity(entity_id).unwrap();

        if self.archetypes[location].is_empty() {
            self.remove_archetype(location);
        }

        // C4/C5: busca posição pelo id, não pelo entity_location
        if let Some(pos) = self.entities.iter().position(|e| e.id == entity_id) {
            self.entities.remove(pos);
        }
    }

    // C7: ordena as keys antes de comparar — HashMap não tem ordem definida
    fn move_entity_to_other_archetype(&mut self, entity: Entity, components: MovedEntity) {
        let mut types_ids = components.keys().copied().collect::<Vec<_>>();
        types_ids.sort();

        let archetype_index = self.archetypes.iter().position(|archetype| {
            let mut arch_keys = archetype.components.keys().copied().collect::<Vec<_>>();
            arch_keys.sort();
            arch_keys == types_ids
        });

        if let Some(archetype_index) = archetype_index {
            self.archetypes[archetype_index].add_entity_migrated(entity.id, components);
            if let Some(e) = self.entities.iter_mut().find(|e| e.id == entity.id) {
                e.entity_location = archetype_index;
            }
        } else {
            let archetype = Archetype::new_from_migration(entity.id, components);
            self.archetypes.push(archetype);
            self.archetype_version += 1;
            let new_idx = self.archetypes.len() - 1;
            if let Some(e) = self.entities.iter_mut().find(|e| e.id == entity.id) {
                e.entity_location = new_idx;
            }
        }
    }

    /// Returns a raw pointer to `entity`'s component `T`, or a [`QueryError`] if either the
    /// entity or the component doesn't exist.
    pub fn get_component<T: 'static + Component>(
        &self,
        entity: Entity,
    ) -> Result<*const T, QueryError> {
        let entity_opt = self.entities.iter().find(|ent| ent.id == entity.id);
        if let Some(entity) = entity_opt {
            let archetype = &self.archetypes[entity.entity_location];
            let component = archetype.get_component::<T>(entity.id);
            match component {
                Some(component) => Ok(component),
                None => Err(QueryError::ComponentNotFound(format!(
                    "Component Type {:?}",
                    std::any::type_name::<T>()
                ))),
            }
        } else {
            Err(QueryError::EntityNotFound(entity.id))
        }
    }

    /// Mutable counterpart to [`EntityManager::get_component`].
    pub fn get_component_mut<T: 'static + Component>(
        &self,
        entity: Entity,
    ) -> Result<*mut T, QueryError> {
        let entity_opt = self.entities.iter().find(|ent| ent.id == entity.id);
        if let Some(entity) = entity_opt {
            let archetype = &self.archetypes[entity.entity_location];
            let component = archetype.get_component_mut::<T>(entity.id);
            match component {
                Some(component) => Ok(component),
                None => Err(QueryError::ComponentNotFound(format!(
                    "Component Type {:?}",
                    std::any::type_name::<T>()
                ))),
            }
        } else {
            Err(QueryError::EntityNotFound(entity.id))
        }
    }

    fn remove_archetype(&mut self, idx: usize) {
        self.archetypes.remove(idx);
        self.archetype_version += 1;
        for entity in self.entities.iter_mut() {
            if entity.entity_location > idx {
                entity.entity_location -= 1;
            }
        }
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod storage_regression_test {
    use super::*;

    #[derive(PartialEq, Eq, Debug)]
    struct A(i32);

    #[derive(PartialEq, Eq, Debug)]
    struct B(i32);

    #[derive(PartialEq, Eq, Debug)]
    struct C(i32);

    #[test]
    fn migration_preserve_other_components_on_remove() {
        let mut em = EntityManager::default();
        let entity = em.create_entity((A(1), B(2), C(3)));
        em.remove_component::<B>(entity);

        let a = em.get_component::<A>(entity).unwrap();
        let c = em.get_component::<C>(entity).unwrap();
        assert_eq!(unsafe { &*a }, &A(1));
        assert_eq!(unsafe { &*c }, &C(3));
        assert!(em.get_component::<B>(entity).is_err());
    }

    #[test]
    fn migration_preserves_other_components_on_add() {
        let mut em = EntityManager::new();
        let entity = em.create_entity((A(1),));

        em.add_component_to_entity(entity, B(2));

        let a = em.get_component::<A>(entity).unwrap();
        let b = em.get_component::<B>(entity).unwrap();
        assert_eq!(unsafe { &*a }, &A(1));
        assert_eq!(unsafe { &*b }, &B(2));
    }

    #[test]
    fn removing_one_entity_does_not_corrupt_siblings() {
        let mut em = EntityManager::new();
        let e1 = em.create_entity((A(1),));
        let e2 = em.create_entity((A(2),));
        let e3 = em.create_entity((A(3),));

        em.remove_entity(e2);

        let a1 = em.get_component::<A>(e1).unwrap();
        let a3 = em.get_component::<A>(e3).unwrap();
        assert_eq!(unsafe { &*a1 }, &A(1));
        assert_eq!(unsafe { &*a3 }, &A(3));
        assert!(em.get_component::<A>(e2).is_err());
    }

    #[test]
    fn removing_middle_entity_of_three_preserves_the_other_two() {
        let mut em = EntityManager::new();
        let e1 = em.create_entity((A(10), B(100)));
        let e2 = em.create_entity((A(20), B(200)));
        let e3 = em.create_entity((A(30), B(300)));

        em.remove_entity(e2);

        assert_eq!(unsafe { &*em.get_component::<A>(e1).unwrap() }, &A(10));
        assert_eq!(unsafe { &*em.get_component::<B>(e1).unwrap() }, &B(100));
        assert_eq!(unsafe { &*em.get_component::<A>(e3).unwrap() }, &A(30));
        assert_eq!(unsafe { &*em.get_component::<B>(e3).unwrap() }, &B(300));
    }

    #[test]
    fn archetype_version_bumps_only_on_new_archetype_shape() {
        let mut em = EntityManager::new();
        let v0 = em.archetype_version;

        em.create_entity((A(1),)); // new shape -> bump
        let v1 = em.archetype_version;
        assert_eq!(v1, v0 + 1);

        em.create_entity((A(2),)); // same shape as an existing archetype -> no bump
        let v2 = em.archetype_version;
        assert_eq!(v2, v1);

        em.create_entity((A(3), B(1))); // new shape -> bump
        let v3 = em.archetype_version;
        assert_eq!(v3, v2 + 1);
    }

    #[test]
    fn archetype_version_bumps_when_an_archetype_is_removed() {
        let mut em = EntityManager::new();
        let e1 = em.create_entity((A(1), B(1)));
        let _e2 = em.create_entity((A(2),)); // second, different shape
        let v_before = em.archetype_version;

        // Removing B leaves e1's old (A, B) archetype empty, which gets pruned.
        em.remove_component::<B>(e1);

        assert!(em.archetype_version > v_before);
    }

    #[test]
    fn query_cache_field_starts_empty_and_round_trips_an_entry() {
        let em = EntityManager::new();
        assert!(em.query_cache.borrow().is_empty());

        let sig = crate::core::query::QuerySignature::new(vec![], vec![], vec![]);
        em.query_cache
            .borrow_mut()
            .insert(sig.clone(), (0, vec![1, 2]));

        assert_eq!(em.query_cache.borrow().get(&sig), Some(&(0, vec![1, 2])));
    }
}
