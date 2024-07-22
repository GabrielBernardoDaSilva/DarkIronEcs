use std::{cell::RefCell, rc::Rc};

use super::{
    component::{BundleComponent, Component},
    entity::Entity,
    entity_manager::EntityManager,
    event::{EventHandler, EventManager},
    query::{Query, QueryParams},
    system::{IntoSystem, SystemManager},
};

pub struct World {
    pub entity_manager: Rc<RefCell<EntityManager>>,
    pub system_manager: Rc<RefCell<SystemManager>>,
    pub event_manager: Rc<RefCell<EventManager>>,
}

impl World {
    pub fn new() -> Self {


         Self {
            entity_manager: Rc::new(RefCell::new(EntityManager::new())),
            system_manager: Rc::new(RefCell::new(SystemManager::new())),
            event_manager: Rc::new(RefCell::new(EventManager::new())),
        }

    

    }

    pub fn create_entity(&mut self, components: impl BundleComponent) -> Entity {
        self.entity_manager.borrow_mut().create_entity(components)
    }

    pub fn remove_component<T: 'static + Component>(&mut self, entity: Entity) {
        self.entity_manager
            .borrow_mut()
            .remove_component::<T>(entity);
    }

    pub fn add_component_to_entity<T: 'static + Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) {
        self.entity_manager
            .borrow_mut()
            .add_component_to_entity(entity, component);
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        self.entity_manager.borrow_mut().remove_entity(entity);
    }

    pub fn create_query<'a, T: QueryParams<'a>>(&'a self) -> Query<'a, T> {
        let entity_manager_ptr = self.entity_manager.as_ptr();
        let mut q = Query::<T>::new(unsafe { &(*entity_manager_ptr).archetypes });
        q.fetch();
        q
    }

    pub(crate) unsafe fn get_entity_manager(&self) -> *const EntityManager {
        self.entity_manager.as_ptr()
    }

    pub fn add_system<P>(&self, system: impl IntoSystem<P>) {
        self.system_manager.borrow_mut().add_system(system);
    }

    pub fn run_systems(&self) {
        self.system_manager.borrow_mut().run_systems(self);
    }

    pub(crate) unsafe fn get_system_manager(&self) -> *const SystemManager {
        self.system_manager.as_ptr()
    }

    pub fn publish_event<T: 'static>(&self, event: T) {
        self.event_manager.borrow_mut().publish(self, event);
    }

    pub fn subscribe_event<T: 'static, FUNC: 'static + Fn(&World, T)>(&self, system: FUNC) {
        let event_handler = EventHandler::new(system);
        self.event_manager.borrow_mut().subscribe(event_handler);
    }

    pub(crate) unsafe fn get_event_manager(&self) -> *const EventManager {
        self.event_manager.as_ptr()
    }
}
