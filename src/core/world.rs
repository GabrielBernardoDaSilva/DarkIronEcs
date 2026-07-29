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

/// The central entry point of the ECS: owns the entity, system, event, resource and
/// coroutine managers and exposes the API used to build and drive an application.
///
/// A `World` is normally created once via [`World::new`] (or [`World::default`]), configured
/// with entities/components/systems, and then driven every frame by calling
/// [`World::run_update`].
///
/// ```
/// # use dark_iron_ecs::core::world::World;
/// let mut world = World::default();
/// world.run_startup();
/// world.run_update();
/// ```
pub struct World {
    /// Owns all entities/components/archetypes. See [`EntityManager`].
    pub entity_manager: Rc<RefCell<EntityManager>>,
    /// Owns registered systems, grouped by [`SystemSchedule`].
    pub system_manager: Rc<RefCell<SystemManager>>,
    /// Owns event subscriptions and dispatches published events. See [`EventManager`].
    pub event_manager: Rc<RefCell<EventManager>>,
    /// Owns global resources. See [`ResourceManager`].
    pub resources: Rc<RefCell<ResourceManager>>,
    /// Owns running coroutines. See [`CoroutineManager`].
    pub coroutine_manager: Rc<RefCell<CoroutineManager>>,
    /// Extensions registered via [`World::add_extension`], applied by [`World::build`].
    pub extensions: Rc<RefCell<Vec<Box<dyn Extension>>>>,
    /// Lightweight handle shared with systems, giving them access to every manager above
    /// without holding a reference to the `World` itself. `None` only before [`World::new`]
    /// finishes constructing it.
    pub coordinator: Option<Rc<RefCell<Coordinator>>>,
}

impl World {
    /// Creates a new, empty `World` with all managers initialized and wired together.
    pub fn new() -> Self {
        let mut world = Self {
            entity_manager: Rc::new(RefCell::new(EntityManager::new())),
            system_manager: Rc::new(RefCell::new(SystemManager::new())),
            event_manager: Rc::new(RefCell::new(EventManager::default())),
            resources: Rc::new(RefCell::new(ResourceManager::new())),
            coroutine_manager: Rc::new(RefCell::new(CoroutineManager::new())),
            extensions: Rc::new(RefCell::new(Vec::new())),
            coordinator: None,
        };

        let coordinator = Rc::new(RefCell::new(Coordinator::new(&world)));
        world
            .event_manager
            .borrow_mut()
            .bind_coordinator(Rc::downgrade(&coordinator));
        world.coordinator = Some(coordinator);

        world
    }

    /// Spawns a new entity with the given bundle of components. Returns `&mut Self` for chaining.
    pub fn create_entity(&mut self, components: impl BundleComponent) -> &mut Self {
        self.entity_manager.borrow_mut().create_entity(components);
        self
    }

    /// Spawns a new entity with the given bundle of components and returns its [`Entity`] id.
    pub fn create_entity_with_id(&mut self, components: impl BundleComponent) -> Entity {
        self.entity_manager.borrow_mut().create_entity(components)
    }

    /// Removes component `T` from `entity`, if present. Returns `&mut Self` for chaining.
    pub fn remove_component<T: 'static + Component>(&mut self, entity: Entity) -> &mut Self {
        self.entity_manager
            .borrow_mut()
            .remove_component::<T>(entity);
        self
    }

    /// Adds (or replaces) component `T` on an existing `entity`. Returns `&mut Self` for chaining.
    pub fn add_component_to_entity<T: 'static + Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> &mut Self {
        self.entity_manager
            .borrow_mut()
            .add_component_to_entity(entity, component);
        self
    }

    /// Removes `entity` and all of its components from the world. Returns `&mut Self` for chaining.
    pub fn remove_entity(&mut self, entity: Entity) -> &mut Self {
        self.entity_manager.borrow_mut().remove_entity(entity);
        self
    }

    /// Builds a [`Query`] over entities that have every component type in `T`.
    pub fn create_query<'a, T: QueryParams<'a>>(&'a self) -> Query<'a, T> {
        let entity_manager = self.entity_manager.clone();
        let archetype_ptr = unsafe { &(*entity_manager.as_ptr()).archetypes };
        Query::<T>::new(Pin::new(archetype_ptr))
    }

    /// Like [`World::create_query`], additionally filtering entities by constraint `C`
    /// (e.g. [`Without`](super::query::Without)).
    pub fn create_query_with_constraint<'a, T: QueryParams<'a>, C: QueryConstraint>(
        &'a self,
    ) -> Query<'a, T, C> {
        let entity_manager = self.entity_manager.clone();
        let archetype_ptr = unsafe { &(*entity_manager.as_ptr()).archetypes };
        Query::<T, C>::new(Pin::new(archetype_ptr))
    }

    /// Registers `system` to run during `system_scheduler`. Returns `&mut Self` for chaining.
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

    /// Registers a tuple of systems to run during `action`. Returns `&mut Self` for chaining.
    pub fn add_systems<P: 'static>(
        &mut self,
        action: SystemSchedule,
        systems: impl SystemBundle<P>,
    ) -> &mut Self {
        systems.add_systems(action, &mut self.system_manager.borrow_mut());
        self
    }

    /// Runs every system registered under [`SystemSchedule::Startup`]. Call once, before the
    /// first [`World::run_update`].
    pub fn run_startup(&mut self) -> &mut Self {
        self.system_manager.borrow_mut().run_startup_systems(self);
        self
    }

    /// Runs every system registered under [`SystemSchedule::Update`]. Call once per frame/tick.
    pub fn run_update(&self) {
        self.system_manager.borrow_mut().run_update_systems(self);
    }

    /// Runs every system registered under [`SystemSchedule::Shutdown`].
    pub fn run_shutdown(&self) {
        self.system_manager.borrow_mut().run_shutdown_systems(self);
    }

    /// Publishes `event` immediately, synchronously invoking whichever handler is subscribed
    /// for type `T`, if any. Returns `&mut Self` for chaining.
    pub fn publish_event<T: 'static>(&mut self, event: T) -> &mut Self {
        let event_manager = self.event_manager.clone();
        event_manager.borrow().publish(event);
        self
    }

    /// Subscribes `system` as the handler for events of type `T`, replacing any previous
    /// handler for that type. Returns `&mut Self` for chaining.
    pub fn subscribe_event<T: 'static, FUNC: 'static + Fn(&World, T)>(
        &mut self,
        system: FUNC,
    ) -> &mut Self {
        let event_handler = EventHandler::new(system);
        self.event_manager.borrow_mut().subscribe(event_handler);
        self
    }

    /// Inserts `resource`, replacing any existing resource of the same type. Returns `&mut Self`
    /// for chaining.
    pub fn add_resource<T: 'static>(&mut self, resource: T) -> &mut Self {
        self.resources.borrow_mut().add(resource);
        self
    }

    /// Returns a handle to the resource of type `T`, or `None` if it hasn't been added.
    pub fn get_resource<T: 'static>(&self) -> Option<Resource<T>> {
        self.resources.borrow().get_resource::<T>()
    }

    /// Starts running `coroutine`. Returns `&mut Self` for chaining.
    pub fn add_coroutine(&mut self, coroutine: Coroutine) -> &mut Self {
        self.coroutine_manager.borrow_mut().add_coroutine(coroutine);
        self
    }

    /// Stops every currently running coroutine.
    pub fn stop_all_coroutines(&self) {
        self.coroutine_manager.borrow_mut().stop_all();
    }

    /// Stops the running coroutine registered under `name`, if any.
    pub fn stop_coroutine_by_name(&self, name: &str) {
        self.coroutine_manager.borrow_mut().stop_by_name(name);
    }

    /// Advances all running coroutines by `delta_time` seconds. Call once per frame.
    pub fn update_coroutines(&mut self, delta_time: f32) {
        let coroutine_manager = self.coroutine_manager.clone();
        coroutine_manager.borrow_mut().update(self, delta_time);
    }

    /// Registers `extension`, to be applied the next time [`World::build`] runs. Returns
    /// `&mut Self` for chaining.
    pub fn add_extension<T: Extension + 'static>(&mut self, extension: T) -> &mut Self {
        let extensions = self.extensions.clone();
        extensions.borrow_mut().push(Box::new(extension));
        self
    }

    /// Runs [`Extension::build`] for every extension registered via [`World::add_extension`].
    /// Returns `&mut Self` for chaining.
    pub fn build(&mut self) -> &mut Self {
        let extensions = self.extensions.clone();
        for extension in extensions.borrow().iter() {
            extension.build(self);
        }
        self
    }

    pub(crate) fn from_coordinator(coordinator: Rc<RefCell<Coordinator>>) -> Self {
        let (entity_manager, system_manager, event_manager, resources, coroutine_manager) = {
            let c = coordinator.borrow();
            (
                c.entity_manager.clone(),
                c.system_manager.clone(),
                c.event_manager.clone(),
                c.resources.clone(),
                c.coroutine_manager.clone(),
            )
        };

        World {
            entity_manager,
            system_manager,
            event_manager,
            resources,
            coroutine_manager,
            extensions: Rc::new(RefCell::new(Vec::new())),
            coordinator: Some(coordinator),
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
