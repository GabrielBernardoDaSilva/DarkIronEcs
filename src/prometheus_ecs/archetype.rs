use std::{cell::RefCell, collections::HashMap};

use super::{
    component::{BundleComponent, Component, ComponentList},
    entity::EntityId,
    error::ArchetypeError,
};

#[derive(Debug)]
pub struct Archetype {
    pub components: HashMap<std::any::TypeId, ComponentList>,
    pub entities: Vec<EntityId>,
}

pub type MovedEntity = HashMap<std::any::TypeId, Box<RefCell<dyn Component>>>;

impl Archetype {
    pub fn new(entity_id: EntityId, components: impl BundleComponent) -> Self {
        let components = components.create_map_components();
        Self {
            components,
            entities: vec![entity_id],
        }
    }
    pub fn new_from_migration(entity_id: EntityId, components: MovedEntity) -> Self {
        let mut components_map = HashMap::new();
        for (type_id, component) in components {
            components_map.insert(type_id, ComponentList { components: vec![component] });
        }
        Self {
            components: components_map,
            entities: vec![entity_id],
        }
    }

    pub fn add_entity(&mut self, entity_id: EntityId, components: impl BundleComponent) {
        for (type_id, component_list) in components.create_map_components() {
            self.components
                .entry(type_id)
                .or_insert_with(ComponentList::new)
                .components
                .extend(component_list.components);
        }
        self.entities.push(entity_id);
    }

    pub fn add_entity_migrated(&mut self, entity_id: EntityId, components: MovedEntity) {
        for (type_id, component) in components {
            self.components
                .entry(type_id)
                .or_insert_with(ComponentList::new)
                .components
                .push(component);
        }
        self.entities.push(entity_id);
    }

    pub fn migrate_entity_to_other_archetype(
        &mut self,
        entity_id: EntityId,
    ) -> Result<(EntityId, MovedEntity), ArchetypeError> {
        let index = self.entities.iter().position(|&x| x == entity_id);
        match index {
            Some(index) => {
                let mut components = HashMap::new();
                for (type_id, component_list) in self.components.iter_mut() {
                    let moved_component = component_list.remove(index);
                    components.insert(*type_id, moved_component);
                }
                self.entities.remove(index);
                Ok((entity_id, components))
            }
            None => Err(ArchetypeError::EntityNotFound),
        }
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) -> Result<(), ArchetypeError> {
        let index = self.entities.iter().position(|&x| x == entity_id);
        match index {
            Some(index) => {
                for component_list in self.components.values_mut() {
                    component_list.remove(index);
                }
                self.entities.remove(index);
                Ok(())
            }
            None => Err(ArchetypeError::EntityNotFound),
        }
    }

    pub fn has_type(&self, type_id: std::any::TypeId) -> bool {
        self.components.contains_key(&type_id)
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
    println!("{:?}", arch.components);

    let (entity_id, moved_entity) = arch.migrate_entity_to_other_archetype(0).unwrap();
    println!("entity {:?}", entity_id);
    println!("{:?}", moved_entity);

    println!("{:?}", arch.components);
}
