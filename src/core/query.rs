use super::access::AccessKey;
use super::archetype::Archetype;
use super::coordinator::Coordinator;
use super::error::QueryError;
use crate::core::system::SystemParam;

use std::any::TypeId;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

/// Implemented for types (and tuples of types, up to 26 elements) that a [`Query`] can fetch —
/// `&T` and `&mut T` for any component `T`. You don't implement this yourself.
pub trait QueryParams<'a> {
    type QueryResult;
    fn get_component_in_archetype(
        archetype: &'a Archetype,
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
    fn constraint_types() -> Vec<TypeId>;
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
    _marked: std::marker::PhantomData<(T, Constraint)>,
}

/// Implemented for `&T` and `&mut T` (any component `T`), letting [`Query`] fetch either
/// shared or exclusive access to a component.
pub trait Fetch<'a> {
    type Result;
    fn fetch(archetype: &'a Archetype, entity_id: u32) -> Result<Self::Result, QueryError>;

    fn get_type_id() -> TypeId;

    /// Whether this fetch requires mutable (`&mut T`) or shared (`&T`) access to the
    /// component.
    fn is_mutable() -> bool;
}

impl<'a, T: 'static> Fetch<'a> for &mut T {
    type Result = Self;
    fn fetch(archetypes: &'a Archetype, entity_id: u32) -> Result<Self::Result, QueryError> {
        let type_id = TypeId::of::<T>();

        match archetypes.components.get(&type_id) {
            Some(res) => match res.get_mut(entity_id as usize) {
                Some(c) => Ok(unsafe { &mut *c }),
                None => Err(QueryError::EntityNotFound(entity_id)),
            },
            None => Err(QueryError::ComponentNotFound(format!(
                "Component Type {:?}",
                std::any::type_name::<T>()
            ))),
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
    fn fetch(archetypes: &'a Archetype, entity_id: u32) -> Result<Self::Result, QueryError> {
        let type_id = TypeId::of::<T>();
        match archetypes.components.get(&type_id) {
            Some(res) => match res.get(entity_id as usize) {
                Some(c) => Ok(unsafe { &*c }),
                None => Err(QueryError::EntityNotFound(entity_id)),
            },
            None => Err(QueryError::ComponentNotFound(format!(
                "Component Type {:?}",
                std::any::type_name::<T>()
            ))),
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

    fn get_component_in_archetype(
        archetype: &'a Archetype,
        entity_location: u32,
    ) -> Option<Self::QueryResult> {
        <T as Fetch>::fetch(archetype, entity_location).ok()
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

macro_rules! impl_query_params {
    ( $head:ident ) => {
        impl<'a, $head: Fetch<'a> + 'static> QueryParams<'a> for ($head,) {
            type QueryResult = $head::Result;

            fn get_component_in_archetype(archetype: &'a Archetype, entity_location: u32) -> Option<Self::QueryResult> {
                $head::fetch(archetype, entity_location).ok()
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
        impl<'a, $head: Fetch<'a>  + 'static, $($tail: Fetch<'a>  + 'static),+> QueryParams<'a> for ($head, $($tail),+) {
            type QueryResult = ($head::Result, $($tail::Result),+);

            fn get_component_in_archetype(archetype: &'a Archetype, entity_location: u32) -> Option<Self::QueryResult> {
                Some((
                    $head::fetch(archetype, entity_location).ok()?,
                    $($tail::fetch(archetype, entity_location).ok()?),+
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

impl_query_params!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

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
            _marked: std::marker::PhantomData,
        }
    }

    /// Runs the query, returning one result per matching entity.
    pub fn fetch(&'a self) -> Vec<<T as QueryParams<'a>>::QueryResult> {
        let types = T::types_id();
        let constraint_types = Constraint::constraint_types();
        let mut components = Vec::new();

        for arch in self.archetypes.iter() {
            let has_any_entities = arch.entities.is_empty();
            let query_needs_more_types_than_archetype_has = types.len() > arch.components.len();
            let contains_all = types
                .iter()
                .all(|type_id| -> bool { arch.has_type(*type_id) });
            let has_constraint = constraint_types
                .iter()
                .any(|type_id| -> bool { arch.has_type(*type_id) });

            if contains_all
                && !has_any_entities
                && !query_needs_more_types_than_archetype_has
                && !has_constraint
            {
                for (index, _) in arch.entities.iter().enumerate() {
                    if let Some(component) = T::get_component_in_archetype(arch, index as u32) {
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
            // A query only reads the archetype layout, never restructures it, so it only
            // needs the entity manager to stay stable — hence a shared (non-mutable) access.
            // This still conflicts with a `&mut EntityManager` taken by the same system,
            // since that could migrate/remove archetypes out from under this query.
            tracker.track(
                AccessKey::Manager(TypeId::of::<super::entity_manager::EntityManager>()),
                false,
                "EntityManager (via Query)",
            );
            for (type_id, mutable) in T::types_id_with_mutability() {
                tracker.track(AccessKey::Component(type_id), mutable, "Query component");
            }
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
