use super::world::World;

/// A reusable bundle of setup logic (entities, resources, systems, ...) that can be registered
/// via [`World::add_extension`](super::world::World::add_extension) and applied later via
/// [`World::build`](super::world::World::build).
pub trait Extension {
    /// Applies this extension's setup to `world`.
    fn build(&self, world: &mut World);
}
