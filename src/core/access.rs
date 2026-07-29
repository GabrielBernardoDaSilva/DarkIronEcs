use std::{any::TypeId, collections::HashMap};

/// Identifies *what* is being accessed by a [`SystemParam`](super::system::SystemParam), so
/// conflicting accesses within a single system call can be detected before they produce two
/// live aliasing references to the same data.
///
/// The three variants are separate namespaces: a `Component(TypeId::of::<Position>())` and a
/// `Resource(TypeId::of::<Position>())` never conflict with each other even though they share
/// a `TypeId`, because nothing stops the same Rust type being used as both a component and a
/// resource.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum AccessKey {
    /// One of the five whole managers reachable via `Coordinator::get_*_mut`.
    Manager(TypeId),
    /// A single component type, as fetched by a [`Query`](super::query::Query).
    Component(TypeId),
    /// A single resource type, as fetched by [`Resource<T>`](super::resources::Resource).
    Resource(TypeId),
}

/// Tracks the accesses made by the [`SystemParam`]s of a single, currently-running system, and
/// panics as soon as two of them would conflict (any pairing where at least one side is
/// mutable). Cleared before each system call by the generated `System::run`.
#[derive(Default)]
pub(crate) struct AccessTracker {
    accesses: HashMap<AccessKey, bool>,
}

impl AccessTracker {
    /// Registers an access to `key`, panicking if it conflicts with one already registered
    /// during this system call.
    pub(crate) fn track(&mut self, key: AccessKey, mutable: bool, type_name: &'static str) {
        if let Some(existing_mutable) = self.accesses.get(&key) {
            if *existing_mutable || mutable {
                panic!(
                    "SystemParam conflict: '{type_name}' is requested more than once by the \
                     same system with incompatible access (at least one of them is mutable). \
                     Combine the parameters or drop the duplicate."
                );
            }
            return;
        }
        self.accesses.insert(key, mutable);
    }

    /// Forgets every access recorded so far, so the next system call starts clean.
    pub(crate) fn clear(&mut self) {
        self.accesses.clear();
    }
}
