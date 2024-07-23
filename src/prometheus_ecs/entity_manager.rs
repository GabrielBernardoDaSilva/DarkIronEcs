use std::cell::RefCell;

use super::{
    archetype::{Archetype, MovedEntity},
    component::{BundleComponent, Component},
    entity::Entity,
    system::SystemParam,
};

pub struct EntityManager {
    pub entities: Vec<Entity>,
    pub archetypes: Vec<Archetype>,
}

impl<'a> SystemParam<'a> for &EntityManager {
    fn get_param(world: &'a super::world::World) -> Self {
        unsafe { &(*world.get_entity_manager()) }
    }
}

impl<'a> SystemParam<'a> for &mut EntityManager { 
    fn get_param(world: &'a super::world::World) -> Self {
        unsafe { &mut (*world.get_entity_manager_mut()) }
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
            let mut arch_types_ids = archetype
                .components
                .keys()
                .map(|key| *key)
                .collect::<Vec<_>>();
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
        let entity = self.entities.iter().find(|ent| ent.id == entity.id);
        match entity {
            Some(entity) => {
                let archetype = &mut self.archetypes[entity.entity_location];
                let type_id = std::any::TypeId::of::<T>();
                let mut entity_with_components = archetype
                    .migrate_entity_to_other_archetype(entity.id)
                    .unwrap();
                entity_with_components.1.remove(&type_id);

                if entity_with_components.1.len() == 0 {
                    self.entities.remove(entity.entity_location);
                } else {
                    self.move_entity_to_other_archetype(*entity, entity_with_components.1);
                }
            }
            None => {}
        }
    }

    pub fn add_component_to_entity<T: 'static + Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) {
        let entity = self.entities.iter().find(|ent| ent.id == entity.id);
        match entity {
            Some(entity) => {
                let archetype = &mut self.archetypes[entity.entity_location];
                let type_id = std::any::TypeId::of::<T>();
                let mut entity_with_components = archetype
                    .migrate_entity_to_other_archetype(entity.id)
                    .unwrap();
                entity_with_components
                    .1
                    .insert(type_id, Box::new(RefCell::new(component)));

                self.move_entity_to_other_archetype(*entity, entity_with_components.1);
            }
            None => {}
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        let entity = self.entities.iter().find(|ent| ent.id == entity.id);
        match entity {
            Some(entity) => {
                let archetype = &mut self.archetypes[entity.entity_location];
                archetype.remove_entity(entity.id).unwrap();
                self.entities.remove(entity.entity_location);
            }
            None => {}
        }
    }

    fn move_entity_to_other_archetype(&mut self, entity: Entity, components: MovedEntity) {
        let types_ids = components.keys().map(|key| *key).collect::<Vec<_>>();
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
}
