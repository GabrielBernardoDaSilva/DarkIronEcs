use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use super::{as_any_trait::AsAny, coordinator::Coordinator, system::SystemParam, world::World};

type EventFunction<T> = Box<dyn Fn(&World, T)>;

pub struct EventHandler<T> {
    pub func: EventFunction<T>,
    _marker: std::marker::PhantomData<T>,
}

pub trait EventTrait: AsAny {}

impl<T> EventHandler<T> {
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

    pub fn subscribe_event<T: 'static, FUNC: 'static + Fn(&World, T)>(
        &mut self,
        event: FUNC,
    ) -> &mut Self {
        let event_handler = EventHandler::new(event);
        self.subscribe(event_handler);
        self
    }

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
        unsafe { &(*coordinator.borrow().get_event_manager_mut()) }
    }
}

impl SystemParam for &mut EventManager {
    fn get_param(coordinator: Rc<RefCell<Coordinator>>) -> Self {
        unsafe { &mut (*coordinator.borrow().get_event_manager_mut()) }
    }
}
