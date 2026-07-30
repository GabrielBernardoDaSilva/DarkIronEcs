use std::{any::TypeId, collections::HashMap};

/// Identifies *what* is being accessed by a [`SystemParam`](super::system::SystemParam), so
/// conflicting accesses within a single system call can be detected before they produce two
/// live aliasing references to the same data.
///
/// [`Query`](super::query::Query) component accesses aren't tracked through this key — see
/// [`AccessTracker::track_query`], which can additionally prove two queries can never match the
/// same entity (e.g. via `Without`) and skip the conflict entirely.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum AccessKey {
    /// One of the five whole managers reachable via `Coordinator::get_*_mut`.
    Manager(TypeId),
    /// A single resource type, as fetched by [`Resource<T>`](super::resources::Resource).
    Resource(TypeId),
}

/// One [`Query`](super::query::Query)'s accesses, as registered with
/// [`AccessTracker::track_query`].
struct QueryAccess {
    /// Component types actually dereferenced by the query, paired with whether the access is
    /// mutable. Overlap here (with incompatible mutability) between two queries is a real
    /// aliasing risk.
    fetched: Vec<(TypeId, bool)>,
    /// Component types required to be present via a `With<T>` filter, but never dereferenced.
    /// Only used to help prove disjointness — never itself a source of conflict.
    present_only: Vec<TypeId>,
    /// Component types required to be *absent* via a `Without<T>` filter.
    excluded: Vec<TypeId>,
}

/// Tracks the accesses made by the [`SystemParam`]s of a single, currently-running system, and
/// panics as soon as two of them would conflict (any pairing where at least one side is
/// mutable). Cleared before each system call by the generated `System::run`.
#[derive(Default)]
pub(crate) struct AccessTracker {
    accesses: HashMap<AccessKey, bool>,
    queries: Vec<QueryAccess>,
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

    /// Registers a [`Query`](super::query::Query)'s accesses, panicking if they conflict with
    /// an already-registered query in this system call — unless the two queries can be proven
    /// to never match the same entity (one excludes, via `Without`, a type the other requires,
    /// either fetched or via `With`), in which case there's no real aliasing risk and no
    /// conflict is raised.
    pub(crate) fn track_query(
        &mut self,
        fetched: Vec<(TypeId, bool)>,
        present_only: Vec<TypeId>,
        excluded: Vec<TypeId>,
        type_name: &'static str,
    ) {
        let required: Vec<TypeId> = fetched
            .iter()
            .map(|(t, _)| *t)
            .chain(present_only.iter().copied())
            .collect();

        for other in &self.queries {
            let other_required: Vec<TypeId> = other
                .fetched
                .iter()
                .map(|(t, _)| *t)
                .chain(other.present_only.iter().copied())
                .collect();

            let provably_disjoint = other_required.iter().any(|t| excluded.contains(t))
                || required.iter().any(|t| other.excluded.contains(t));
            if provably_disjoint {
                continue;
            }

            for (type_id, mutable) in &fetched {
                if let Some((_, other_mutable)) =
                    other.fetched.iter().find(|(t, _)| t == type_id)
                    && (*mutable || *other_mutable)
                {
                    panic!(
                        "SystemParam conflict: Query<{type_name}> conflicts with another \
                         Query in the same system over a shared component. Add a \
                         `Without<...>` filter proving the two queries never match the same \
                         entity, or split into separate systems."
                    );
                }
            }
        }

        self.queries.push(QueryAccess {
            fetched,
            present_only,
            excluded,
        });
    }

    /// Forgets every access recorded so far, so the next system
    pub(crate) fn clear(&mut self) {
        self.accesses.clear();
        self.queries.clear();
    }
}
