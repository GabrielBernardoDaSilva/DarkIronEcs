use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use super::{system::SystemParam, world::World};

pub struct EventHandler<T> {
    pub func: Box<dyn Fn(&World, T)>,
    _marker: std::marker::PhantomData<T>,
}

pub trait EventTrait {
    fn to_any(&self) -> &dyn Any;
}

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

impl<T: 'static> EventTrait for EventHandler<T> {
    fn to_any(&self) -> &dyn Any {
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

    pub fn publish<T: 'static>(&mut self, world: &World, t: T) {
        let event = self.events.get_mut(&TypeId::of::<T>());
        match event {
            Some(event) => {
                let event_handler = event.to_any().downcast_ref::<EventHandler<T>>().unwrap();
                event_handler.call(world, t);
            }
            None => {}
        }
    }
}

impl<'a> SystemParam<'a> for &EventManager {
    fn get_param(world: &'a World) -> Self {
        unsafe { &(*world.get_event_manager()) }
    }
}
