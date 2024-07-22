use super::world::World;

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
    ($($param:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        impl<'a, Func, $($param,)*> System<($($param,)*)> for Func
        where
            Func: Fn($($param),*),
            $($param: SystemParam<'a>,)*
        {
            fn run(&self, world: &World) {
                $(
                    let ptr = unsafe {std::mem::transmute::<&World, &World>(world)};
                    let $param = $param::get_param(ptr);
                )*
                self($($param,)*);
            }
        }
    };
}

impl_system!(A);
impl_system!(A, B);
impl_system!(A, B, C);
impl_system!(A, B, C, D);
impl_system!(A, B, C, D, E);
impl_system!(A, B, C, D, E, F);
impl_system!(A, B, C, D, E, F, G);
impl_system!(A, B, C, D, E, F, G, H);
impl_system!(A, B, C, D, E, F, G, H, I);
impl_system!(A, B, C, D, E, F, G, H, I, J);
impl_system!(A, B, C, D, E, F, G, H, I, J, K);
impl_system!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_system!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_system!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_system!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);

pub struct SystemManager {
    pub systems: Vec<Box<dyn FnMut(&World)>>,
}

impl SystemManager {
    pub fn new() -> Self {
        SystemManager {
            systems: Vec::new(),
        }
    }

    pub fn add_system<P, F>(&mut self, system: F)
    where
        F: IntoSystem<P>,
    {
        self.systems.push(system.system());
    }

    pub fn run_systems(&mut self, world: &World) {
        for system in self.systems.iter_mut() {
            system(world);
        }
    }
}

impl<'a> SystemParam<'a> for &SystemManager {
    fn get_param(world: &'a World) -> Self {
        unsafe { &(*world.get_system_manager()) }
    }
}
