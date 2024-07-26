use super::{system::SystemParam, world::World};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct WaitAmountOfSeconds {
    pub amount_in_seconds: f32,
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum CoroutineState {
    Running,
    Yielded(WaitAmountOfSeconds),
    Finished,
}

pub struct Coroutine {
    name: String,
    state: CoroutineState,
    generator: Box<dyn FnMut(&mut World) -> CoroutineState + 'static>,
    is_waiting: bool,
    amount_to_wait: f32,
}

impl Coroutine {
    // Constructor to create a new coroutine
    pub fn new(name: &str, generator: impl FnMut(&mut World) -> CoroutineState + 'static) -> Self {
        Self {
            name: name.to_owned(),
            state: CoroutineState::Running,
            generator: Box::new(generator),
            is_waiting: false,
            amount_to_wait: 0.0,
        }
    }

    // Function to resume execution of the coroutine
    fn resume(&mut self, world: &mut World) -> Option<WaitAmountOfSeconds> {
        match self.state {
            CoroutineState::Running => {
                let next_state = (self.generator)(world);
                self.state = next_state;
                self.resume(world)
            }
            CoroutineState::Yielded(value) => {
                self.state = CoroutineState::Running;
                Some(value)
            }
            CoroutineState::Finished => {
                self.state = CoroutineState::Finished;
                None
            }
        }
    }

    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        if self.is_waiting {
            self.amount_to_wait -= delta_time;

            if self.amount_to_wait > 0.0 {
                return;
            }
            self.is_waiting = false;
        }

        if let Some(res) = self.resume(world) {
            self.is_waiting = true;
            self.amount_to_wait = res.amount_in_seconds;
        }
    }

    pub fn stop(&mut self) {
        self.state = CoroutineState::Finished;
    }
}

pub struct CoroutineManager {
    coroutines: Vec<Coroutine>,
}

impl CoroutineManager {
    pub fn new() -> Self {
        Self {
            coroutines: Vec::new(),
        }
    }

    pub fn add_coroutine(&mut self, coroutine: Coroutine) {
        self.coroutines.push(coroutine);
    }

    pub fn update(&mut self, world: &mut World, delta_time: f32) {
        for thread in self.coroutines.iter_mut() {
            if thread.state == CoroutineState::Finished {
                continue;
            }
            thread.update(world, delta_time);
        }

        self.coroutines
            .retain(|thread| thread.state != CoroutineState::Finished);
    }

    pub fn stop_all(&mut self) {
        self.coroutines.iter_mut().for_each(|thread| thread.stop());
    }

    pub fn stop_by_name(&mut self, name: &str) {
        let coroutine = self
            .coroutines
            .iter_mut()
            .find(|thread| thread.name == name);
        if let Some(soul_thread) = coroutine {
            soul_thread.stop();
        }
    }
}

impl Default for CoroutineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SystemParam<'a> for &CoroutineManager {
    fn get_param(world: &'a World) -> Self {
        unsafe {
            let coroutine_manager = world.get_coroutine_manager();
            &*coroutine_manager
        }
    }
}

impl<'a> SystemParam<'a> for &mut CoroutineManager {
    fn get_param(world: &'a World) -> Self {
        unsafe {
            let coroutine_manager = world.get_coroutine_manager_mut();
            &mut *coroutine_manager
        }
    }
}
