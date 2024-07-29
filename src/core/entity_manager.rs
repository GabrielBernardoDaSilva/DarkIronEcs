use std::cell::UnsafeCell;

use super::{
    archetype::{Archetype, MovedEntity},
    component::{BundleComponent, Component},
    entity::Entity,
    error::QueryError,
    system::SystemParam,
};

pub struct EntityManager {
    pub entities: Vec<Entity>,
    pub archetypes: Vec<Archetype>,
}

impl SystemParam for &EntityManager {
    fn get_param(
        coordinator: std::rc::Rc<std::cell::RefCell<super::coordinator::Coordinator>>,
    ) -> Self {
        unsafe { &*coordinator.borrow().get_entity_manager() }
    }
}

impl SystemParam for &mut EntityManager {
    fn get_param(
        coordinator: std::rc::Rc<std::cell::RefCell<super::coordinator::Coordinator>>,
    ) -> Self {
        unsafe { &mut *coordinator.borrow().get_entity_manager_mut() }
    }
}

impl EntityManager {
    pub fn new() -> Self {
        EntityManager {
            entities: Vec::new(),
            archetypes: Vec::new(),
        }
    }

    pub fn create_entity(&mut self, components: impl BundleComponent) -> Entity {
        let mut types_ids = components.get_types_id();
        let mut entity = Entity::new(self.entities.len() as u32, 0);

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
                entity.entity_location = self.archetypes.len() - 1;
            }
        }

        self.entities.push(entity);
        entity
    }

    pub fn remove_component<T: 'static + Component>(&mut self, entity: Entity) {
        let entity_opt = self.entities.iter().find(|ent| ent.id == entity.id);

        if let Some(entity) = entity_opt {
            let location = entity.entity_location;
            let archetype = &mut self.archetypes[location];
            let type_id = std::any::TypeId::of::<T>();
            let mut entity_with_components = archetype
                .migrate_entity_to_other_archetype(entity.id)
                .unwrap();
            entity_with_components.1.remove(&type_id);

            if archetype.is_empty() {
                self.archetypes.remove(location);
            }

            if entity_with_components.1.is_empty() {
                self.entities.remove(location);
            } else {
                self.move_entity_to_other_archetype(*entity, entity_with_components.1);
            }
        }
    }

    pub fn add_component_to_entity<T: 'static + Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) {
        let entity_opt = self.entities.iter().find(|ent| ent.id == entity.id);
        if let Some(entity) = entity_opt {
            let archetype = &mut self.archetypes[entity.entity_location];
            let type_id = std::any::TypeId::of::<T>();
            let mut entity_with_components = archetype
                .migrate_entity_to_other_archetype(entity.id)
                .unwrap();
            entity_with_components
                .1
                .insert(type_id, Box::new(UnsafeCell::new(component)));

            if archetype.is_empty() {
                self.archetypes.remove(entity.entity_location);
            }

            self.move_entity_to_other_archetype(*entity, entity_with_components.1);
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        let entity_opt = self.entities.iter().find(|ent| ent.id == entity.id);
        if let Some(entity) = entity_opt {
            let archetype = &mut self.archetypes[entity.entity_location];
            archetype.remove_entity(entity.id).unwrap();
            if archetype.is_empty() {
                self.archetypes.remove(entity.entity_location);
            }
            self.entities.remove(entity.entity_location);
        }
    }

    fn move_entity_to_other_archetype(&mut self, entity: Entity, components: MovedEntity) {
        let types_ids = components.keys().copied().collect::<Vec<_>>();
        let archetype_index = self
            .archetypes
            .iter()
            .position(|archetype| archetype.components.keys().eq(types_ids.iter()));

        if let Some(archetype_index) = archetype_index {
            self.archetypes[archetype_index].add_entity_migrated(entity.id, components);
            self.entities[entity.entity_location].entity_location = archetype_index;
        } else {
            let archetype = Archetype::new_from_migration(entity.id, components);
            self.archetypes.push(archetype);
            self.entities[entity.entity_location].entity_location = self.archetypes.len() - 1;
        }
    }

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
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}
