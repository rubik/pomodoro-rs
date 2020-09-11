use std::sync::{Arc, Mutex};
use crate::state::PomodoroState;

pub struct PomodoroClock {
    state: Arc<Mutex<PomodoroState>>,
}

impl PomodoroClock {
    pub fn new(state: Arc<Mutex<PomodoroState>>) -> Self {
        Self {
            state,
        }
    }
}
