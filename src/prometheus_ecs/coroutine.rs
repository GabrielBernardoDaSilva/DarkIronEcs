use super::world::World;

pub enum CoroutineState {
    Running,
    Waiting,
    Finished,
}

pub enum CoroutineExecution {
    Once,
    Loop,
    Repeat(usize),
}

pub struct WaitAmountOfSeconds {
    pub seconds: f64,
    pub current_time: f64,
}

pub struct Coroutine {
    pub wait_time: WaitAmountOfSeconds,
    pub state: CoroutineState,
    pub execution: CoroutineExecution,
    pub function: Box<dyn FnMut(&mut World)>,
}

impl Coroutine {
    pub fn new(function: impl FnMut(&mut World) + 'static, execution: CoroutineExecution, mut wait_time: WaitAmountOfSeconds) -> Self {
        wait_time.current_time = 0.0;
        Self {
            wait_time,
            function: Box::new(function),
            state: CoroutineState::Running,
            execution,
        }
    }

    pub fn run(&mut self, world: &mut World, delta_time: f64) {
        self.wait_time.current_time += delta_time;
        
        
    }
}
