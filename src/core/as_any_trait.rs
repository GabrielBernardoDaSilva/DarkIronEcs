use std::any::Any;

/// Lets a trait object be downcast back to its concrete type via `as_any().downcast_ref`.
/// Implemented by [`EventHandler`](super::event::EventHandler) and [`Resource`](super::resources::Resource)
/// so [`EventManager`](super::event::EventManager) and [`ResourceManager`](super::resources::ResourceManager)
/// can store heterogeneous, type-erased values in one map and recover the original type on
/// lookup.
pub trait AsAny {
    /// Returns `self` as `&dyn Any`.
    fn as_any(&self) -> &dyn Any;
    /// Returns `self` as `&mut dyn Any`.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
