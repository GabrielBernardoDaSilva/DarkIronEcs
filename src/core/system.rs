use std::collections::HashMap;

use super::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemSchedule {
    Startup,
    Update,
    Shutdown,
}

pub trait SystemParam<'a> {
    fn get_param(world: &'a World) -> Self;
}

pub trait System<P> {
    fn run(&self, world: &World);
}

pub trait IntoSystem<P> {
    fn system(self) -> Box<dyn FnMut(&World)>;
}

impl<F, P> IntoSystem<P> for F
where
    F: System<P> + 'static,
{
    fn system(self) -> Box<dyn FnMut(&World)> {
        Box::new(move |world| self.run(world))
    }
}


macro_rules! impl_system {
    ( $head:ident ) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<'a, Func, $head> System<($head,)> for Func
        where
            Func: Fn($head),
            $head: SystemParam<'a>,
        {
            fn run(&self, world: &World) {
                let ptr = world as *const World;
                let $head = $head::get_param(unsafe { &*ptr });
                self($head);
            }
        }


    };
    // Recursive case: Implement for tuples with more than one element
    ( $head:ident, $($tail:ident),+ ) => {
        impl_system!($($tail),+);

        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<'a, Func, $head, $($tail,)*> System<($head, $($tail,)*)> for Func
        where
            Func: Fn($head, $($tail),*),
            $head: SystemParam<'a>,
            $($tail: SystemParam<'a>,)*
        {
            fn run(&self, world: &World) {
                let ptr = world as *const World;
                let $head = $head::get_param(unsafe { &*ptr });
                $(
                    let $tail = $tail::get_param(unsafe { &*ptr });
                )*
                self($head, $($tail),*);
            }
        }
    }
}

impl_system!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);



type SystemFunctionMap = HashMap<SystemSchedule, Vec<Box<dyn FnMut(&World)>>>;

pub struct SystemManager {
    pub systems: SystemFunctionMap,
}

impl SystemManager {
    pub fn new() -> Self {
        SystemManager {
            systems: HashMap::new(),
        }
    }

    pub fn add_system<P, F>(&mut self, system_schedule: SystemSchedule, system: F)
    where
        F: IntoSystem<P>,
    {
        self.systems
            .entry(system_schedule)
            .or_default()
            .push(system.system());
    }

    pub fn run_startup_systems(&mut self, world: &World) {
        if let Some(systems) = self.systems.get_mut(&SystemSchedule::Startup) {
            for system in systems.iter_mut() {
                system(world);
            }
        }
    }

    pub fn run_update_systems(&mut self, world: &World) {
        if let Some(systems) = self.systems.get_mut(&SystemSchedule::Update) {
            for system in systems.iter_mut() {
                system(world);
            }
        }
    }

    pub fn run_shutdown_systems(&mut self, world: &World) {
        if let Some(systems) = self.systems.get_mut(&SystemSchedule::Shutdown) {
            for system in systems.iter_mut() {
                system(world);
            }
        }
    }
}

impl<'a> SystemParam<'a> for &SystemManager {
    fn get_param(world: &'a World) -> Self {
        unsafe { &(*world.get_system_manager()) }
    }
}

pub trait SystemBundle<P> {
    fn add_systems(self, system_schedule: SystemSchedule, system_manager: &mut SystemManager);
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! impl_system_bundle {

    ( ($head:ident, $identifier:ident) ) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<$identifier: 'static, $head: IntoSystem<$identifier>> SystemBundle<($identifier,)> for ($head,) {
            fn add_systems(self, system_schedule: SystemSchedule, system_manager: &mut SystemManager) {
                system_manager.add_system(system_schedule, self.0);
            }
        }
    };

    ( ($head:ident, $identifier:ident), $( ($tail:ident, $identifier_tail:ident) ),* ) => {

        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<$identifier: 'static, $head: IntoSystem<$identifier>, $($identifier_tail: 'static, $tail: IntoSystem<$identifier_tail>),*> SystemBundle<($identifier, $($identifier_tail),*)> for ($head, $($tail),*)
        where
            $($tail: IntoSystem<$identifier_tail>,)*
        {
            fn add_systems(self, system_schedule: SystemSchedule, system_manager: &mut SystemManager) {

                let ($head, $($tail),*) = self;

                system_manager.add_system(system_schedule, $head);
                $(
                    system_manager.add_system(system_schedule, $tail);
                )*
            }
        }

        impl_system_bundle!($(($tail, $identifier_tail)),*);
    };
}

impl_system_bundle!(
    (A, A1),
    (B, B1),
    (C, C1),
    (D, D1),
    (E, E1),
    (F, F1),
    (G, G1),
    (H, H1),
    (I, I1),
    (J, J1),
    (K, K1),
    (L, L1),
    (M, M1),
    (N, N1),
    (O, O1),
    (P, P1),
    (Q, Q1),
    (R, R1),
    (S, S1),
    (T, T1),
    (U, U1),
    (V, V1),
    (W, W1),
    (X, X1),
    (Y, Y1),
    (Z, Z1)
);
