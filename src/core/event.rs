use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
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

pub struct EventManager {
    pub events: HashMap<TypeId, Box<dyn EventTrait>>,
}

impl EventManager {
    pub fn new() -> Self {
        EventManager {
            events: HashMap::new(),
        }
    }

    pub fn subscribe<T: 'static>(&mut self, event: EventHandler<T>) {
        self.events.insert(TypeId::of::<T>(), Box::new(event));
    }

    pub fn publish<T: 'static>(&mut self, w: &World, t: T) {
        let event_opt = self.events.get_mut(&TypeId::of::<T>());

        if let Some(event) = event_opt {
            let event_handler = event.as_any().downcast_ref::<EventHandler<T>>().unwrap();
            event_handler.call(w, t);
        }
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

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}
