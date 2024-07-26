use std::{cell::RefCell, rc::Rc};

use super::{
    coroutine::CoroutineManager, entity_manager::EntityManager, event::EventManager,
    resources::ResourceManager, system::SystemManager, world::World,
};

pub struct Coordinator {
    pub entity_manager: Rc<RefCell<EntityManager>>,
    pub system_manager: Rc<RefCell<SystemManager>>,
    pub event_manager: Rc<RefCell<EventManager>>,
    pub resources: Rc<RefCell<ResourceManager>>,
    pub coroutine_manager: Rc<RefCell<CoroutineManager>>,
}

impl Coordinator {
    pub fn new(world: &World) -> Self {
        Self {
            entity_manager: world.entity_manager.clone(),
            system_manager: world.system_manager.clone(),
            event_manager: world.event_manager.clone(),
            resources: world.resources.clone(),
            coroutine_manager: world.coroutine_manager.clone(),
        }
    }

    pub(crate) unsafe fn get_entity_manager(&self) -> *const EntityManager {
        self.entity_manager.as_ptr()
    }

    pub(crate) unsafe fn get_entity_manager_mut(&self) -> *mut EntityManager {
        self.entity_manager.as_ptr()
    }

    pub(crate) unsafe fn get_system_manager(&self) -> *const SystemManager {
        self.system_manager.as_ptr()
    }

    pub(crate) unsafe fn get_event_manager(&self) -> *const EventManager {
        self.event_manager.as_ptr()
    }

    pub(crate) unsafe fn get_event_manager_mut(&self) -> *mut EventManager {
        self.event_manager.as_ptr()
    }

    pub(crate) unsafe fn get_resource_manager(&self) -> *const ResourceManager {
        self.resources.as_ptr()
    }

    pub(crate) unsafe fn get_resource_manager_mut(&self) -> *mut ResourceManager {
        self.resources.as_ptr()
    }

    pub(crate) unsafe fn get_coroutine_manager(&self) -> *const CoroutineManager {
        self.coroutine_manager.as_ptr()
    }

    pub(crate) unsafe fn get_coroutine_manager_mut(&self) -> *mut CoroutineManager {
        self.coroutine_manager.as_ptr()
    }
}
