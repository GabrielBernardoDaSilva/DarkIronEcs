use std::{
    any::TypeId,
    cell::RefCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use super::{access::AccessKey, as_any_trait::AsAny, coordinator::Coordinator, system::SystemParam};

/// Object-safe marker implemented by every [`Resource<T>`], letting [`ResourceManager`] store
/// resources of different types in a single `HashMap` and downcast them back via [`AsAny`].
pub trait ResourceTrait: AsAny {}

/// A shared, reference-counted handle to a single global value of type `T`, dereferencing to
/// `&T`/`&mut T`. Obtained via [`World::get_resource`](super::world::World::get_resource) or
/// as a system parameter.
pub struct Resource<T: ?Sized> {
    value: *const T,
    pub type_id: std::any::TypeId,
    counter: Rc<RefCell<u32>>,
}

impl<T: 'static> ResourceTrait for Resource<T> {}

impl<T: 'static> Resource<T> {
    /// Wraps `value` as a resource. Usually not called directly — prefer
    /// [`World::add_resource`](super::world::World::add_resource).
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Resource {
            value: ptr,
            type_id: std::any::TypeId::of::<T>(),
            counter: Rc::new(RefCell::new(1)),
        }
    }
}

impl<T: 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        let counter = self.counter.clone();
        *counter.borrow_mut() += 1;
        Resource {
            value: self.value,
            type_id: self.type_id,
            counter,
        }
    }
}

impl<T: ?Sized> Drop for Resource<T> {
    fn drop(&mut self) {
        unsafe {
            if *self.counter.borrow() > 1 {
                *self.counter.borrow_mut() -= 1;
                return;
            }
            let value = self.value as *mut T;
            drop(Box::from_raw(value));
        }
    }
}

impl<T: 'static> Deref for Resource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<T: 'static> DerefMut for Resource<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.value as *mut T) }
    }
}

impl<T: 'static> AsAny for Resource<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<T: 'static> std::fmt::Display for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource<{}>", std::any::type_name::<T>())
    }
}

impl<T: 'static> SystemParam for Resource<T> {
    fn get_param(coordinator: Rc<RefCell<Coordinator>>) -> Self {
        // Always tracked as mutable: `Resource<T>` always exposes `DerefMut`, so a second
        // handle to the same `T` in this system could alias a live `&mut T` from this one.
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Resource(TypeId::of::<T>()),
            true,
            std::any::type_name::<T>(),
        );
        let resource_manager = coordinator.borrow().resources.clone();
        let resource = resource_manager.borrow().get_resource::<T>().unwrap();
        resource.clone()
    }
}

/// Owns every registered global [`Resource`], keyed by type. Most callers interact with it
/// indirectly through [`World`](super::world::World) rather than directly.
pub struct ResourceManager {
    pub resources: HashMap<TypeId, Rc<dyn ResourceTrait>>,
}

impl ResourceManager {
    /// Creates an empty manager with no resources.
    pub fn new() -> Self {
        ResourceManager {
            resources: HashMap::new(),
        }
    }

    /// Inserts `resource`, replacing any existing resource of the same type.
    pub fn add<T: 'static>(&mut self, resource: T) {
        let res = Resource::new(resource);
        self.resources.insert(TypeId::of::<T>(), Rc::new(res));
    }

    /// Returns a handle to the resource of type `T`, or `None` if it hasn't been added.
    pub fn get_resource<T: 'static>(&self) -> Option<Resource<T>> {
        let resource = self.resources.get(&TypeId::of::<T>())?;
        let resource = resource.as_any().downcast_ref::<Resource<T>>()?;
        Some(resource.clone())
    }
}

impl SystemParam for &ResourceManager {
    fn get_param(coordinator: Rc<RefCell<Coordinator>>) -> Self {
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Manager(TypeId::of::<ResourceManager>()),
            false,
            "ResourceManager",
        );
        unsafe { &(*coordinator.borrow().get_resource_manager_mut()) }
    }
}

impl SystemParam for &mut ResourceManager {
    fn get_param(coordinator: Rc<RefCell<Coordinator>>) -> Self {
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Manager(TypeId::of::<ResourceManager>()),
            true,
            "ResourceManager",
        );
        unsafe { &mut (*coordinator.borrow().get_resource_manager_mut()) }
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
