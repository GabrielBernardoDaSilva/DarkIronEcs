use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use super::{
    access::AccessKey, as_any_trait::AsAny, coordinator::Coordinator, system::SystemParam,
    world::World,
};

type EventFunction<T> = Box<dyn Fn(&World, T)>;

/// A subscribed callback for events of type `T`, wrapping a `Fn(&World, T)` closure.
///
/// Usually built indirectly via [`World::subscribe_event`] or [`EventManager::subscribe_event`]
/// rather than constructed directly.
pub struct EventHandler<T> {
    pub func: EventFunction<T>,
    _marker: std::marker::PhantomData<T>,
}

/// Object-safe marker implemented by every [`EventHandler<T>`], letting [`EventManager`] store
/// handlers for different event types in a single `HashMap` and downcast them back via
/// [`AsAny`].
pub trait EventTrait: AsAny {}

impl<T> EventHandler<T> {
    /// Wraps `func` as an event handler.
    pub fn new(func: impl Fn(&World, T) + 'static) -> Self {
        Self {
            func: Box::new(func),
            _marker: std::marker::PhantomData,
        }
    }

    fn call(&self, world: &World, t: T) {
        (self.func)(world, t);
    }
}

impl<T: 'static> EventTrait for EventHandler<T> {}

impl<T: 'static> AsAny for EventHandler<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Owns event subscriptions (one handler per event type) and dispatches published events.
///
/// Available as a [`SystemParam`] (`&EventManager` or `&mut EventManager`), so systems can
/// subscribe and publish without needing access to [`World`] directly — see
/// [`EventManager::publish`].
#[derive(Default)]
pub struct EventManager {
    pub events: HashMap<TypeId, Box<dyn EventTrait>>,
    coordinator: Option<Weak<RefCell<Coordinator>>>,
}

impl EventManager {
    pub(crate) fn subscribe<T: 'static>(&mut self, event: EventHandler<T>) {
        self.events.insert(TypeId::of::<T>(), Box::new(event));
    }

    fn publish_with_world<T: 'static>(&self, w: &World, t: T) {
        let event_opt = self.events.get(&TypeId::of::<T>());

        if let Some(event) = event_opt {
            let event_handler = event.as_any().downcast_ref::<EventHandler<T>>().unwrap();
            event_handler.call(w, t);
        }
    }

    pub(crate) fn bind_coordinator(&mut self, coordinator: Weak<RefCell<Coordinator>>) {
        self.coordinator = Some(coordinator);
    }

    /// Subscribes `event` as the handler for events of type `T`, replacing any previous
    /// handler for that type. Returns `&mut Self` for chaining.
    pub fn subscribe_event<T: 'static, FUNC: 'static + Fn(&World, T)>(
        &mut self,
        event: FUNC,
    ) -> &mut Self {
        let event_handler = EventHandler::new(event);
        self.subscribe(event_handler);
        self
    }

    /// Publishes `t` immediately, synchronously invoking whichever handler is subscribed for
    /// type `T`, if any. No-op if nothing is subscribed to `T`.
    ///
    /// Unlike [`World::publish_event`], this doesn't need a `&World` — it reconstructs one
    /// internally from the bound [`Coordinator`] so it can be called from systems that only
    /// have access to `&EventManager`/`&mut EventManager`.
    ///
    /// # Panics
    /// Panics if called before [`World::new`] finished binding this manager to its
    /// `Coordinator`, or if the owning `World` has since been dropped.
    pub fn publish<T: 'static>(&self, t: T) {
        let coordinator = self.coordinator.clone().expect(
            "EventManager not bound to Coordinator - publish() called before world new completed",
        )
        .upgrade().expect("Coordinator dropped");
        let world = World::from_coordinator(coordinator);
        self.publish_with_world(&world, t);
    }
}

impl SystemParam for &EventManager {
    fn get_param(coordinator: Rc<RefCell<Coordinator>>) -> Self {
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Manager(TypeId::of::<EventManager>()),
            false,
            "EventManager",
        );
        unsafe { &(*coordinator.borrow().get_event_manager_mut()) }
    }
}

impl SystemParam for &mut EventManager {
    fn get_param(coordinator: Rc<RefCell<Coordinator>>) -> Self {
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Manager(TypeId::of::<EventManager>()),
            true,
            "EventManager",
        );
        unsafe { &mut (*coordinator.borrow().get_event_manager_mut()) }
    }
}
