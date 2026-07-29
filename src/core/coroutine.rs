use super::{access::AccessKey, system::SystemParam, world::World};

/// A pause duration yielded from a coroutine body via [`CoroutineState::Yielded`].
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct WaitAmountOfSeconds {
    pub amount_in_seconds: f32,
}

/// The result of resuming a [`Coroutine`] for one step: keep running immediately, pause for a
/// duration, or stop for good.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum CoroutineState {
    Running,
    Yielded(WaitAmountOfSeconds),
    Finished,
}

/// A named, resumable task driven by [`World::update_coroutines`](super::world::World::update_coroutines)
/// once per frame. Its body is a closure returning a [`CoroutineState`] each time it's resumed,
/// letting it pause execution for a number of seconds via `CoroutineState::Yielded`.
pub struct Coroutine {
    name: String,
    state: CoroutineState,
    generator: Box<dyn FnMut(&mut World) -> CoroutineState + 'static>,
    is_waiting: bool,
    amount_to_wait: f32,
}

impl Coroutine {
    /// Creates a coroutine named `name`, running `generator` as its body. Panics later if
    /// added to a [`CoroutineManager`] that already has a coroutine with the same name.
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

    /// Advances the coroutine by `delta_time` seconds: resumes it once any pending wait has
    /// elapsed, running the body until it yields a new wait or finishes.
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

    /// Marks the coroutine as finished; it will be dropped on the next [`CoroutineManager::update`].
    pub fn stop(&mut self) {
        self.state = CoroutineState::Finished;
    }
}

/// Owns and drives every running [`Coroutine`]. Most callers interact with it indirectly
/// through [`World`] rather than directly.
pub struct CoroutineManager {
    coroutines: Vec<Coroutine>,
}

impl CoroutineManager {
    /// Creates an empty manager with no coroutines running.
    pub fn new() -> Self {
        Self {
            coroutines: Vec::new(),
        }
    }

    /// Starts running `coroutine`.
    ///
    /// # Panics
    /// Panics if a coroutine with the same name is already running.
    pub fn add_coroutine(&mut self, coroutine: Coroutine) {
        if self.coroutines.iter().any(|c| c.name == coroutine.name) {
            panic!("Coroutine with name '{}' already exists", coroutine.name);
        }
        self.coroutines.push(coroutine);
    }

    /// Advances every running coroutine by `delta_time` seconds, dropping any that finished.
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

    /// Stops every currently running coroutine.
    pub fn stop_all(&mut self) {
        self.coroutines.iter_mut().for_each(|thread| thread.stop());
    }

    /// Stops the running coroutine named `name`, if any.
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

impl SystemParam for &CoroutineManager {
    fn get_param(
        coordinator: std::rc::Rc<std::cell::RefCell<super::coordinator::Coordinator>>,
    ) -> Self {
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Manager(std::any::TypeId::of::<CoroutineManager>()),
            false,
            "CoroutineManager",
        );
        unsafe { &(*coordinator.borrow().get_coroutine_manager_mut()) }
    }
}

impl SystemParam for &mut CoroutineManager {
    fn get_param(
        coordinator: std::rc::Rc<std::cell::RefCell<super::coordinator::Coordinator>>,
    ) -> Self {
        coordinator.borrow().access_tracker.borrow_mut().track(
            AccessKey::Manager(std::any::TypeId::of::<CoroutineManager>()),
            true,
            "CoroutineManager",
        );
        unsafe { &mut (*coordinator.borrow().get_coroutine_manager_mut()) }
    }
}
