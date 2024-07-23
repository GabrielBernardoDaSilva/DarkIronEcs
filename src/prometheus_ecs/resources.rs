use std::ops::{Deref, DerefMut};

use super::as_any_trait::AsAny;

pub trait ResourceTrait: AsAny {}

pub struct Resource<T: ?Sized> {
    pub value: *const T,
    pub type_id: std::any::TypeId,
}

impl<T: 'static> Resource<T> {
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Resource {
            value: ptr,
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}

impl<T: 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Resource {
            value: self.value,
            type_id: self.type_id,
        }
    }
}

impl<T: ?Sized> Drop for Resource<T> {
    fn drop(&mut self) {
        unsafe {
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



pub struct ResourceManager {
   
}   