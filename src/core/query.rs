use super::archetype::Archetype;
use super::coordinator::Coordinator;
use super::error::QueryError;
use crate::core::system::SystemParam;

use std::any::TypeId;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

pub trait QueryParams<'a> {
    type QueryResult;
    fn get_component_in_archetype(
        archetype: &'a Archetype,
        entity_location: u32,
    ) -> Self::QueryResult;

    fn types_id() -> Vec<TypeId>;
}

pub trait QueryConstraint {
    fn constraint_types() -> Vec<TypeId>;
}

impl QueryConstraint for () {
    fn constraint_types() -> Vec<TypeId> {
        Vec::new()
    }
}

pub trait Constraints {
    fn constraint_types() -> Vec<TypeId>;
}

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

pub struct Query<'a, T: QueryParams<'a> + 'static, Constraint: QueryConstraint = ()> {
    pub archetypes: Pin<&'a Vec<Archetype>>,
    _marked: std::marker::PhantomData<(T, Constraint)>,
}
pub trait Fetch<'a> {
    type Result;
    fn fetch(archetype: &'a Archetype, entity_id: u32) -> Result<Self::Result, QueryError>;

    fn get_type_id() -> TypeId;
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
}

impl<'a, T: Fetch<'a> + 'static> QueryParams<'a> for T {
    type QueryResult = T::Result;

    fn get_component_in_archetype(
        archetype: &'a Archetype,
        entity_location: u32,
    ) -> Self::QueryResult {
        match <T as Fetch<'a>>::fetch(archetype, entity_location) {
            Ok(res) => res,
            Err(e) => panic!("{:?}", e),
        }
    }

    fn types_id() -> Vec<TypeId> {
        vec![<T>::get_type_id()]
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

            fn get_component_in_archetype(archetype: &'a Archetype, entity_location: u32) -> Self::QueryResult {
                match $head::fetch(archetype, entity_location){
                    Ok(res) => res,
                    Err(e) => panic!("{:?}", e)
                }
            }

            fn types_id() -> Vec<TypeId> {
                vec![<$head>::get_type_id()]
            }
        }



    };
    ( $head:ident, $($tail:ident),+ ) => {
        impl<'a, $head: Fetch<'a>  + 'static, $($tail: Fetch<'a>  + 'static),+> QueryParams<'a> for ($head, $($tail),+) {
            type QueryResult = ($head::Result, $($tail::Result),+);

            fn get_component_in_archetype(archetype: &'a Archetype, entity_location: u32) -> Self::QueryResult {
                (
                    match $head::fetch(archetype, entity_location){
                        Ok(res) => res,
                        Err(e) => panic!("{:?}", e)
                    },
                    $(
                        match $tail::fetch(archetype, entity_location){
                            Ok(res) => res,
                            Err(e) => panic!("{:?}", e)
                        }
                    ),+
                )
            }
            fn types_id() -> Vec<TypeId> {
                let types = vec![<$head>::get_type_id(), $($tail::get_type_id()),+];
                types
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
    pub fn new(archetypes: Pin<&'a Vec<Archetype>>) -> Query<'a, T, Constraint> {
        Query {
            archetypes,
            _marked: std::marker::PhantomData,
        }
    }
    #[deprecated]
    pub fn iter(&'a self) -> Vec<<T as QueryParams<'a>>::QueryResult> {
        let types = T::types_id();
        let constraint_types = Constraint::constraint_types();
        let mut components = Vec::new();

        for arch in self.archetypes.iter() {
            let has_any_entities = arch.entities.is_empty();
            let is_archetype_components_bigger = types.len() > arch.components.len();
            let contains_all = types
                .iter()
                .all(|type_id| -> bool { arch.has_type(*type_id) });
            let has_constraint = constraint_types
                .iter()
                .any(|type_id| -> bool { arch.has_type(*type_id) });

            if contains_all
                && !has_any_entities
                && !is_archetype_components_bigger
                && !has_constraint
            {
                for (index, _) in arch.entities.iter().enumerate() {
                    let component = T::get_component_in_archetype(arch, index as u32);
                    components.push(component);
                }
            }
        }
        components
    }

    pub fn fetch(&'a self) -> Vec<<T as QueryParams<'a>>::QueryResult> {
        let types = T::types_id();
        let constraint_types = Constraint::constraint_types();
        let mut components = Vec::new();

        for arch in self.archetypes.iter() {
            let has_any_entities = arch.entities.is_empty();
            let is_archetype_components_bigger = types.len() > arch.components.len();
            let contains_all = types
                .iter()
                .all(|type_id| -> bool { arch.has_type(*type_id) });
            let has_constraint = constraint_types
                .iter()
                .any(|type_id| -> bool { arch.has_type(*type_id) });

            if contains_all
                && !has_any_entities
                && !is_archetype_components_bigger
                && !has_constraint
            {
                for (index, _) in arch.entities.iter().enumerate() {
                    let component = T::get_component_in_archetype(arch, index as u32);
                    components.push(component);
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
