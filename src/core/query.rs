use super::access::AccessKey;
use super::archetype::Archetype;
use super::coordinator::Coordinator;
use super::error::QueryError;
use crate::core::component::ComponentList;
use crate::core::system::SystemParam;

use std::any::TypeId;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

/// Implemented for types (and tuples of types, up to 26 elements) that a [`Query`] can fetch —
/// `&T` and `&mut T` for any component `T`. You don't implement this yourself.
pub trait QueryParams<'a> {
    type QueryResult;

    /// The resolved per-archetype data this query reads from — computed once per archetype by
    /// [`QueryParams::resolve`], then indexed per entity by
    /// [`QueryParams::get_component_from_source`], instead of re-resolving on every entity.
    type Source;

    /// Resolves this query's `ComponentList`(s) for `archetype`. Returns `None` if the
    /// archetype is missing a required type.
    fn resolve(archetype: &'a Archetype) -> Option<Self::Source>;

    /// Reads this query's result for one entity from an already-`resolve`d `Source`.
    fn get_component_from_source(
        source: &Self::Source,
        entity_location: u32,
    ) -> Option<Self::QueryResult>;

    fn types_id() -> Vec<TypeId>;

    /// Like [`QueryParams::types_id`], paired with whether each type is fetched mutably
    /// (`&mut T`) or not (`&T`). Used to detect conflicting `SystemParam` accesses.
    fn types_id_with_mutability() -> Vec<(TypeId, bool)>;
}

/// Filter applied to a [`Query`], excluding entities that match. The default `()` applies no
/// filtering; use [`Without`] to exclude entities that have a given component.
pub trait QueryConstraint {
    /// Types that must be absent for an entity to match.
    fn constraint_types() -> Vec<TypeId>;
    /// Types that must be present (but aren't fetched) for an entity to match. Defaults to
    /// none, so existing `Without`-only constraints don't need to change.
    fn required_types() -> Vec<TypeId> {
        Vec::new()
    }
}

impl QueryConstraint for () {
    fn constraint_types() -> Vec<TypeId> {
        Vec::new()
    }
}

/// Implemented for types (and tuples of types) usable inside [`Without<T>`].
pub trait Constraints {
    fn constraint_types() -> Vec<TypeId>;
}

/// A [`Query`] constraint that excludes entities having any of the component types in `T`.
///
/// ```
/// # use dark_iron_ecs::core::query::{Query, Without};
/// # struct Health(i32);
/// # struct Name(String);
/// fn system(q: Query<(&Health,), Without<(&Name,)>>) {
///     for health in q.fetch() {
///         println!("no Name — {}", health.0);
///     }
/// }
/// ```
pub struct Without<T: Constraints + 'static>(std::marker::PhantomData<T>);

impl<T: Constraints> QueryConstraint for Without<T> {
    fn constraint_types() -> Vec<TypeId> {
        T::constraint_types()
    }
}

impl Constraints for () {
    fn constraint_types() -> Vec<TypeId> {
        Vec::new()
    }
}

/// A [`Query`] constraint that requires entities to have every component type in `T`, without
/// fetching their values.
///
/// ```
/// # use dark_iron_ecs::core::query::{Query, With};
/// # struct Position(f32, f32);
/// # struct Player;
/// fn system(q: Query<(&Position,), With<&Player>>) {
///     for position in q.fetch() {
///         println!("player at {}, {}", position.0, position.1);
///     }
/// }
/// ```
pub struct With<T: Constraints + 'static>(std::marker::PhantomData<T>);

impl<T: Constraints> QueryConstraint for With<T> {
    fn constraint_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn required_types() -> Vec<TypeId> {
        T::constraint_types()
    }
}

/// Reads entities that have every component type in `T` (and none of the types excluded by
/// `Constraint`, see [`Without`]). Take one as a system parameter, or build one manually via
/// [`World::create_query`](super::world::World::create_query).
///
/// ```
/// # use dark_iron_ecs::core::query::Query;
/// # struct Health(i32);
/// fn system(q: Query<(&Health,)>) {
///     for health in q.fetch() {
///         println!("Health: {}", health.0);
///     }
/// }
/// ```
pub struct Query<'a, T: QueryParams<'a> + 'static, Constraint: QueryConstraint = ()> {
    pub archetypes: Pin<&'a Vec<Archetype>>,
    types: Vec<TypeId>,
    excluded_types: Vec<TypeId>,
    required_types: Vec<TypeId>,
    _marked: std::marker::PhantomData<(T, Constraint)>,
}

/// Implemented for `&T` and `&mut T` (any component `T`), letting [`Query`] fetch either
/// shared or exclusive access to a component.
pub trait Fetch<'a> {
    type Result;

    /// Resolves the `ComponentList` this fetch reads from, once per archetype instead of once
    /// per entity. Returns `None` if the archetype doesn't have this component type.
    fn resolve(archetype: &'a Archetype) -> Option<&'a ComponentList>;

    /// Reads this fetch's result for one entity from an already-`resolve`d `ComponentList`.
    fn fetch_from(list: &'a ComponentList, entity_id: u32) -> Result<Self::Result, QueryError>;

    fn get_type_id() -> TypeId;

    /// Whether this fetch requires mutable (`&mut T`) or shared (`&T`) access to the
    /// component.
    fn is_mutable() -> bool;
}

impl<'a, T: 'static> Fetch<'a> for &mut T {
    type Result = Self;

    fn resolve(archetype: &'a Archetype) -> Option<&'a ComponentList> {
        archetype.components.get(&TypeId::of::<T>())
    }

    fn fetch_from(list: &'a ComponentList, entity_id: u32) -> Result<Self::Result, QueryError> {
        match list.get_mut(entity_id as usize) {
            Some(c) => Ok(unsafe { &mut *c }),
            None => Err(QueryError::EntityNotFound(entity_id)),
        }
    }

    fn get_type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn is_mutable() -> bool {
        true
    }
}

impl<'a, T: 'static> Fetch<'a> for &T {
    type Result = Self;

    fn resolve(archetype: &'a Archetype) -> Option<&'a ComponentList> {
        archetype.components.get(&TypeId::of::<T>())
    }

    fn fetch_from(list: &'a ComponentList, entity_id: u32) -> Result<Self::Result, QueryError> {
        match list.get(entity_id as usize) {
            Some(c) => Ok(unsafe { &*c }),
            None => Err(QueryError::EntityNotFound(entity_id)),
        }
    }

    fn get_type_id() -> TypeId {
        TypeId::of::<T>()
    }

    fn is_mutable() -> bool {
        false
    }
}

impl<'a, T: Fetch<'a> + 'static> QueryParams<'a> for T {
    type QueryResult = T::Result;
    type Source = &'a ComponentList;

    fn resolve(archetype: &'a Archetype) -> Option<Self::Source> {
        <T as Fetch>::resolve(archetype)
    }

    fn get_component_from_source(
        source: &Self::Source,
        entity_location: u32,
    ) -> Option<Self::QueryResult> {
        <T as Fetch>::fetch_from(source, entity_location).ok()
    }

    fn types_id() -> Vec<TypeId> {
        vec![<T>::get_type_id()]
    }

    fn types_id_with_mutability() -> Vec<(TypeId, bool)> {
        vec![(<T>::get_type_id(), <T>::is_mutable())]
    }
}

impl<T: for<'a> Fetch<'a> + 'static> Constraints for T {
    fn constraint_types() -> Vec<TypeId> {
        vec![<T>::get_type_id()]
    }
}

// Used only inside `impl_query_params!` to give every element of a tuple the same `Source`
// type (`&'a ComponentList`) while still referencing `$tail`/`$head` in the projection, which
// macro_rules requires to know how many times to repeat.
#[doc(hidden)]
pub trait Resolved<'a> {
    type Source;
}

impl<'a, F: Fetch<'a>> Resolved<'a> for F {
    type Source = &'a ComponentList;
}

macro_rules! impl_query_params {
    ( $head:ident ) => {
        impl<'a, $head: Fetch<'a> + 'static> QueryParams<'a> for ($head,) {
            type QueryResult = $head::Result;
            type Source = &'a ComponentList;

            fn resolve(archetype: &'a Archetype) -> Option<Self::Source> {
                <$head as Fetch>::resolve(archetype)
            }

            fn get_component_from_source(
                source: &Self::Source,
                entity_location: u32,
            ) -> Option<Self::QueryResult> {
                <$head as Fetch>::fetch_from(source, entity_location).ok()
            }

            fn types_id() -> Vec<TypeId> {
                vec![<$head>::get_type_id()]
            }

            fn types_id_with_mutability() -> Vec<(TypeId, bool)> {
                vec![(<$head>::get_type_id(), <$head>::is_mutable())]
            }
        }



    };
    ( $head:ident, $($tail:ident),+ ) => {
        #[allow(non_snake_case)]
        impl<'a, $head: Fetch<'a>  + 'static, $($tail: Fetch<'a>  + 'static),+> QueryParams<'a> for ($head, $($tail),+) {
            type QueryResult = ($head::Result, $($tail::Result),+);
            type Source = (<$head as Resolved<'a>>::Source, $(<$tail as Resolved<'a>>::Source),+);

            fn resolve(archetype: &'a Archetype) -> Option<Self::Source> {
                Some((
                    <$head as Fetch>::resolve(archetype)?,
                    $(<$tail as Fetch>::resolve(archetype)?),+
                ))
            }

            fn get_component_from_source(
                source: &Self::Source,
                entity_location: u32,
            ) -> Option<Self::QueryResult> {
                let ($head, $($tail),+) = source;
                Some((
                    <$head as Fetch>::fetch_from($head, entity_location).ok()?,
                    $(<$tail as Fetch>::fetch_from($tail, entity_location).ok()?),+
                ))
            }

            fn types_id() -> Vec<TypeId> {
                let types = vec![<$head>::get_type_id(), $($tail::get_type_id()),+];
                types
            }

            fn types_id_with_mutability() -> Vec<(TypeId, bool)> {
                vec![
                    (<$head>::get_type_id(), <$head>::is_mutable()),
                    $((<$tail>::get_type_id(), <$tail>::is_mutable())),+
                ]
            }
        }


        impl_query_params!($($tail),+);
    };
}

impl_query_params!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);

macro_rules! impl_query_constrains {
    ( $head:ident ) => {
        impl<$head: for<'a> Fetch<'a>  + 'static > Constraints for ($head,) {
            fn constraint_types() -> Vec<TypeId> {
                vec![<$head>::get_type_id()]
            }
        }
    };
    ( $head:ident, $($tail:ident),+ ) => {
        impl<$head: for<'a> Fetch<'a>  + 'static, $($tail: for<'a> Fetch<'a> + 'static),+> Constraints for ($head, $($tail),+) {
            fn constraint_types() -> Vec<TypeId> {
                vec![<$head>::get_type_id(), $($tail::get_type_id()),+]
            }
        }
        impl_query_constrains!($($tail),+);
    };
}

impl_query_constrains!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);

impl<'a, T: QueryParams<'a> + 'static, Constraint: QueryConstraint> Query<'a, T, Constraint> {
    /// Builds a query over the given archetype list. Usually not called directly — prefer
    /// [`World::create_query`](super::world::World::create_query) or taking a `Query` as a
    /// system parameter.
    pub fn new(archetypes: Pin<&'a Vec<Archetype>>) -> Query<'a, T, Constraint> {
        Query {
            archetypes,
            types: T::types_id(),
            excluded_types: Constraint::constraint_types(),
            required_types: Constraint::required_types(),
            _marked: std::marker::PhantomData,
        }
    }

    /// Runs the query, returning one result per matching entity.
    pub fn fetch(&'a self) -> Vec<<T as QueryParams<'a>>::QueryResult> {
        let mut components = Vec::new();

        for arch in self.archetypes.iter() {
            let has_any_entities = arch.entities.is_empty();
            let query_needs_more_types_than_archetype_has =
                self.types.len() > arch.components.len();
            let has_constraint = self
                .excluded_types
                .iter()
                .any(|type_id| -> bool { arch.has_type(*type_id) });
            let is_missing = self
                .required_types
                .iter()
                .any(|type_id| -> bool { !arch.has_type(*type_id) });

            if has_any_entities
                || query_needs_more_types_than_archetype_has
                || has_constraint
                || is_missing
            {
                continue;
            }

            // Resolves every `ComponentList` this query needs *once* for this archetype,
            // instead of re-resolving them on every entity below.
            if let Some(source) = T::resolve(arch) {
                for (index, _) in arch.entities.iter().enumerate() {
                    if let Some(component) = T::get_component_from_source(&source, index as u32) {
                        components.push(component);
                    }
                }
            }
        }
        components
    }
}

impl<'a, T: QueryParams<'a>, Constraint: QueryConstraint + 'static> SystemParam
    for Query<'a, T, Constraint>
{
    fn get_param(coordinator: Rc<RefCell<Coordinator>>) -> Self {
        {
            let coordinator_ref = coordinator.borrow();
            let mut tracker = coordinator_ref.access_tracker.borrow_mut();
            tracker.track(
                AccessKey::Manager(TypeId::of::<super::entity_manager::EntityManager>()),
                false,
                "EntityManager (via Query)",
            );
            tracker.track_query(
                T::types_id_with_mutability(),
                Constraint::required_types(),
                Constraint::constraint_types(),
                std::any::type_name::<T>(),
            );
        }
        let entity_manager: Rc<RefCell<super::entity_manager::EntityManager>> =
            coordinator.borrow().entity_manager.clone();
        let ptr = entity_manager.as_ptr();
        let archetypes = Pin::new(unsafe { &(*ptr).archetypes });
        Query::<T, Constraint>::new(archetypes)
    }
}

#[test]
fn query_test() {
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Health(i32);
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Position(i32, i32);
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Velocity(i32, i32);
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Name(String);
    let v = vec![];
    let q = Query::<&Health>::new(Pin::new(&v));
    for h in q.fetch() {
        println!("{:?}", h);
    }
}
