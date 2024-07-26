use super::archetype::Archetype;
use crate::core::system::SystemParam;
use crate::core::world::World;
use std::any::TypeId;

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
    pub archetypes: &'a Vec<Archetype>,
    _marked: std::marker::PhantomData<(T, Constraint)>,
}
pub trait Fetch<'a> {
    type Result;
    fn fetch(archetype: &'a Archetype, entity_id: u32) -> Self::Result;

    fn get_type_id() -> TypeId;
}

impl<'a, T: 'static> Fetch<'a> for &mut T {
    type Result = Self;
    fn fetch(archetypes: &'a Archetype, entity_id: u32) -> Self::Result {
        let type_id = TypeId::of::<T>();

        let res = archetypes.components.get(&type_id).unwrap();
        let c: &mut T = res.get_mut(entity_id as usize).unwrap();
        return unsafe {
            let ptr = c as *mut T;
            &mut *ptr
        };
    }

    fn get_type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

impl<'a, T: 'static> Fetch<'a> for &T {
    type Result = Self;
    fn fetch(archetypes: &'a Archetype, entity_id: u32) -> Self::Result {
        let type_id = TypeId::of::<T>();
        let res = archetypes.components.get(&type_id).unwrap();
        let c: &mut T = res.get_mut(entity_id as usize).unwrap();
        return unsafe {
            let ptr = c as *mut T;
            &mut *ptr
        };
    }

    fn get_type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

macro_rules! impl_query_params {
    ( $head:ident ) => {
        impl<'a, $head: Fetch<'a> + 'static> QueryParams<'a> for ($head,) {
            type QueryResult = $head::Result;

            fn get_component_in_archetype(archetype: &'a Archetype, entity_location: u32) -> Self::QueryResult {
                $head::fetch(archetype, entity_location)
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
                ($head::fetch(archetype, entity_location), $($tail::fetch(archetype, entity_location)),+)
            }
            fn types_id() -> Vec<TypeId> {
                let mut types = Vec::new();
                types.push(<$head>::get_type_id());
                $(types.push(<$tail>::get_type_id());)+
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
    pub fn new(world: &'a World) -> Self {
        let archetypes = unsafe { &(*world.entity_manager.as_ptr()).archetypes };

        Query {
            archetypes,
            _marked: std::marker::PhantomData,
        }
    }

    pub fn iter(&self) -> Vec<<T as QueryParams<'a>>::QueryResult> {
        let types = T::types_id();
        let constraint_types = Constraint::constraint_types();
        let mut components = Vec::new();

        for arch in self.archetypes.iter() {
            let has_any_entities = arch.entities.is_empty();
            let is_archetype_components_bigger = types.len() > arch.components.len();
            let contains_all = types
                .iter()
                .any(|type_id| -> bool { arch.has_type(*type_id) });
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
        return components;
    }
}

impl<'a, T: QueryParams<'a>, Constraint: QueryConstraint + 'static> SystemParam<'a>
    for Query<'a, T, Constraint>
{
    fn get_param(world: &'a World) -> Self {
       Query::<T, Constraint>::new(world)
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

}
