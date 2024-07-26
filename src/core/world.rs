use std::{cell::RefCell, pin::Pin, rc::Rc};

use super::{
    component::{BundleComponent, Component},
    coordinator::Coordinator,
    coroutine::{Coroutine, CoroutineManager},
    entity::Entity,
    entity_manager::EntityManager,
    event::{EventHandler, EventManager},
    extension::Extension,
    query::{Query, QueryConstraint, QueryParams},
    resources::{Resource, ResourceManager},
    system::{IntoSystem, SystemBundle, SystemManager, SystemSchedule},
};

pub struct World {
    pub entity_manager: Rc<RefCell<EntityManager>>,
    pub system_manager: Rc<RefCell<SystemManager>>,
    pub event_manager: Rc<RefCell<EventManager>>,
    pub resources: Rc<RefCell<ResourceManager>>,
    pub coroutine_manager: Rc<RefCell<CoroutineManager>>,
    pub extensions: Rc<RefCell<Vec<Box<dyn Extension>>>>,
    pub coordinator: Option<Rc<RefCell<Coordinator>>>,
}

impl World {
    pub fn new() -> Self {
        let mut world = Self {
            entity_manager: Rc::new(RefCell::new(EntityManager::new())),
            system_manager: Rc::new(RefCell::new(SystemManager::new())),
            event_manager: Rc::new(RefCell::new(EventManager::new())),
            resources: Rc::new(RefCell::new(ResourceManager::new())),
            coroutine_manager: Rc::new(RefCell::new(CoroutineManager::new())),
            extensions: Rc::new(RefCell::new(Vec::new())),
            coordinator: None,
        };
        world.event_manager.borrow_mut().set_world(&world);

        let coordinator = Coordinator::new(&world);
        world.coordinator = Some(Rc::new(RefCell::new(coordinator)));

        world
    }

    pub fn create_entity(&mut self, components: impl BundleComponent) -> &mut Self {
        self.entity_manager.borrow_mut().create_entity(components);
        self
    }

    pub fn create_entity_with_id(&mut self, components: impl BundleComponent) -> Entity {
        self.entity_manager.borrow_mut().create_entity(components)
    }

    pub fn remove_component<T: 'static + Component>(&mut self, entity: Entity) -> &mut Self {
        self.entity_manager
            .borrow_mut()
            .remove_component::<T>(entity);
        self
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

    pub fn create_query<'a, T: QueryParams<'a>>(&'a self) -> Query<T> {
        let entity_manager = self.entity_manager.clone();
        let archetype_ptr = unsafe { &(*entity_manager.as_ptr()).archetypes };
        Query::<T>::new(Pin::new(archetype_ptr))
    }

    pub fn create_query_with_constraint<'a, T: QueryParams<'a>, C: QueryConstraint>(
        &'a self,
    ) -> Query<T, C> {
        let entity_manager = self.entity_manager.clone();
        let archetype_ptr = unsafe { &(*entity_manager.as_ptr()).archetypes };
        Query::<T, C>::new(Pin::new(archetype_ptr))
    }

    pub fn add_system<P>(
        &mut self,
        system_scheduler: SystemSchedule,
        system: impl IntoSystem<P>,
    ) -> &mut Self {
        self.system_manager
            .borrow_mut()
            .add_system(system_scheduler, system);
        self
    }

    pub fn add_systems<P: 'static>(
        &mut self,
        action: SystemSchedule,
        systems: impl SystemBundle<P>,
    ) -> &mut Self {
        let system_manager = unsafe { &mut (*self.get_system_manager_mut()) };
        systems.add_systems(action, system_manager);
        self
    }
    pub fn run_startup(&self) -> &Self {
        self.system_manager.borrow_mut().run_startup_systems(self);
        self
    }

    pub fn run_update(&self) {
        self.system_manager.borrow_mut().run_update_systems(self);
    }

    pub fn run_shutdown(&self) {
        self.system_manager.borrow_mut().run_shutdown_systems(self);
    }

    pub(crate) unsafe fn get_system_manager_mut(&self) -> *mut SystemManager {
        self.system_manager.as_ptr()
    }

    pub fn publish_event<T: 'static>(&self, event: T) {
        self.event_manager.borrow_mut().publish(event);
    }

    pub fn subscribe_event<T: 'static, FUNC: 'static + Fn(&World, T)>(&self, system: FUNC) {
        let event_handler = EventHandler::new(system);
        self.event_manager.borrow_mut().subscribe(event_handler);
    }
    pub fn add_resource<T: 'static>(&self, resource: T) {
        self.resources.borrow_mut().add(resource);
    }

    pub fn get_resource<T: 'static>(&self) -> Option<Resource<T>> {
        self.resources.borrow().get_resource::<T>()
    }

    pub fn add_coroutine(&self, coroutine: Coroutine) {
        self.coroutine_manager.borrow_mut().add_coroutine(coroutine);
    }

    pub fn stop_all_coroutines(&self) {
        self.coroutine_manager.borrow_mut().stop_all();
    }

    pub fn stop_coroutine_by_name(&self, name: &str) {
        self.coroutine_manager.borrow_mut().stop_by_name(name);
    }

    pub fn update_coroutines(&mut self, delta_time: f32) {
        let coroutine_manager = self.coroutine_manager.clone();
        coroutine_manager.borrow_mut().update(self, delta_time);
    }

    pub fn add_extension<T: Extension + 'static>(&mut self, extension: T) {
        let extensions = self.extensions.clone();
        extensions.borrow_mut().push(Box::new(extension));
    }

    pub fn build(&mut self) {
        let extensions = self.extensions.clone();
        for extension in extensions.borrow().iter() {
            extension.build(self);
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
