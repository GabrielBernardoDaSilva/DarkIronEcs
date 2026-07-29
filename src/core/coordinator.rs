use std::{cell::RefCell, rc::Rc};

use super::{
    access::AccessTracker, coroutine::CoroutineManager, entity_manager::EntityManager,
    event::EventManager, resources::ResourceManager, system::SystemManager, world::World,
};

/// A lightweight, cloneable handle to every manager owned by a [`World`], passed to systems
/// instead of the `World` itself so a system's parameters (see [`SystemParam`](super::system::SystemParam))
/// can each fetch just the manager they need.
pub struct Coordinator {
    pub entity_manager: Rc<RefCell<EntityManager>>,
    pub system_manager: Rc<RefCell<SystemManager>>,
    pub event_manager: Rc<RefCell<EventManager>>,
    pub resources: Rc<RefCell<ResourceManager>>,
    pub coroutine_manager: Rc<RefCell<CoroutineManager>>,
    pub(crate) access_tracker: RefCell<AccessTracker>,
}

impl Coordinator {
    /// Builds a `Coordinator` sharing `world`'s managers. Called once by
    /// [`World::new`](super::world::World::new); not normally constructed directly.
    pub fn new(world: &World) -> Self {
        Self {
            entity_manager: world.entity_manager.clone(),
            system_manager: world.system_manager.clone(),
            event_manager: world.event_manager.clone(),
            resources: world.resources.clone(),
            coroutine_manager: world.coroutine_manager.clone(),
            access_tracker: RefCell::new(AccessTracker::default()),
        }
    }

    pub(crate) unsafe fn get_entity_manager_mut(&self) -> *mut EntityManager {
        self.entity_manager.as_ptr()
    }

    pub(crate) unsafe fn get_system_manager_mut(&self) -> *mut SystemManager {
        self.system_manager.as_ptr()
    }

    pub(crate) unsafe fn get_event_manager_mut(&self) -> *mut EventManager {
        self.event_manager.as_ptr()
    }

    pub(crate) unsafe fn get_resource_manager_mut(&self) -> *mut ResourceManager {
        self.resources.as_ptr()
    }

    pub(crate) unsafe fn get_coroutine_manager_mut(&self) -> *mut CoroutineManager {
        self.coroutine_manager.as_ptr()
    }
}
