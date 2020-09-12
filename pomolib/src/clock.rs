use crate::state::{PomodoroState, TransitionResult};
use futures::future::{abortable, AbortHandle};
use std::sync::{Arc, Mutex};
use tokio::time;

pub struct PomodoroClock {
    state: Arc<Mutex<PomodoroState>>,
    abort_handle: Option<AbortHandle>,
}

impl PomodoroClock {
    pub fn new(state: Arc<Mutex<PomodoroState>>) -> Self {
        Self {
            state,
            abort_handle: None,
        }
    }

    pub fn start(&mut self, s: u64) {
        let state = self.state.clone();
        let (task, abort_handle) = abortable(async move {
            let mut wait_sec = s;
            loop {
                time::delay_for(time::Duration::from_secs(wait_sec)).await;
                match state.lock().unwrap().transition() {
                    TransitionResult::Stopped => break,
                    TransitionResult::NextTransitionIn(s) => {
                        wait_sec = s.into()
                    }
                }
            }
        });
        self.abort_handle = Some(abort_handle);
        tokio::spawn(task);
    }

    pub fn stop(&mut self) {
        if let Some(ah) = self.abort_handle.take() {
            ah.abort();
        }
    }
}
