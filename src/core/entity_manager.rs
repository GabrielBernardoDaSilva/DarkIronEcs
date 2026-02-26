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
    next_entity_id: u32, // L6: contador monotônico — evita colisão de IDs após remoções
}

impl SystemParam for &EntityManager {
    fn get_param(
        coordinator: std::rc::Rc<std::cell::RefCell<super::coordinator::Coordinator>>,
    ) -> Self {
        unsafe { &*coordinator.borrow().get_entity_manager_mut() }
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
            next_entity_id: 0,
        }
    }

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
                entity.entity_location = self.archetypes.len() - 1;
            }
        }

        self.entities.push(entity);
        entity
    }

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
        entity_with_components
            .1
            .insert(type_id, Box::new(UnsafeCell::new(component)));

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
            let new_idx = self.archetypes.len() - 1;
            if let Some(e) = self.entities.iter_mut().find(|e| e.id == entity.id) {
                e.entity_location = new_idx;
            }
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

    fn remove_archetype(&mut self, idx: usize) {
        self.archetypes.remove(idx);
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
