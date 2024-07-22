use super::archetype::Archetype;
use crate::prometheus_ecs::system::SystemParam;
use crate::prometheus_ecs::world::World;
use std::{any::TypeId, fmt::Debug};

pub trait QueryParams<'a> {
    type QueryResult;
    fn get_component_in_archetype(
        archetype: &'a Archetype,
        entity_location: u32,
    ) -> Self::QueryResult;

    fn types_id() -> Vec<TypeId>;
}

pub struct Query<'a, T: QueryParams<'a> + 'static> {
    pub components: Vec<<T as QueryParams<'a>>::QueryResult>,
    pub archetypes: &'a Vec<Archetype>,
    _marked: std::marker::PhantomData<T>,
}
pub trait Fetch<'a> {
    type Result;
    fn fetch(archetype: &'a Archetype, entity_id: u32) -> Self::Result;

    fn get_type_id() -> TypeId;
}

impl<'a, T: 'static + Debug> Fetch<'a> for &mut T {
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

impl<'a, T: 'static + Debug> Fetch<'a> for &T {
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
        impl<'a, $head: Fetch<'a> + Debug + 'static> QueryParams<'a> for ($head,) {
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
        impl<'a, $head: Fetch<'a> + Debug + 'static, $($tail: Fetch<'a> + Debug + 'static),+> QueryParams<'a> for ($head, $($tail),+) {
            type QueryResult = ($head::Result, $($tail::Result),+);

            fn get_component_in_archetype(archetype: &'a Archetype, entity_location: u32) -> Self::QueryResult {
                ($head::fetch(archetype, 0), $($tail::fetch(archetype, entity_location)),+)
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

impl<'a, T: QueryParams<'a> + 'static> Query<'a, T> {
    pub fn new(archetypes: &'a Vec<Archetype>) -> Self {
        Query {
            components: Vec::new(),
            archetypes,
            _marked: std::marker::PhantomData,
        }
    }

    pub fn fetch(&mut self) {
        let types = T::types_id();
        for arch in self.archetypes.iter() {
            let mut contains_all = true;
            for type_id in types.iter() {
                if !arch.has_type(*type_id) {
                    contains_all = false;
                    continue;
                }
            }
            if !contains_all {
                continue;
            }
            for (index, _) in arch.entities.iter().enumerate() {
                let component = T::get_component_in_archetype(arch, index as u32);
                self.components.push(component);
            }
        }
    }
}

impl<'a, T: QueryParams<'a>> SystemParam<'a> for Query<'a, T> {
    fn get_param(world: &'a World) -> Self {
        world.create_query::<T>()
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

    let mut arch = Archetype::new(0, (Health(100), Position(0, 0), Velocity(0, 0)));
    arch.add_entity(1, (Health(200), Position(1, 1), Velocity(1, 1)));
    let v = vec![arch];

    let mut q = Query::<(&Health, &Velocity, &Position)>::new(&v);
    let _q1 = Query::<(&mut Health,)>::new(&v);
    // let _q2 = Query::<(&mut Health, &mut Velocity)>::new(&v);

    q.fetch();

    for (health, velocity, position) in q.components {
        println!("{:?} {:?} {:?}", health, velocity, position);
    }

    // q.fetch();

    // let a = q.components;
    // for (health, position, velocity) in a {
    //     println!("{:?} {:?} {:?}", health, position, velocity);
    //     // health.0 = 1000;

    // }

    // q.fetch();
}
