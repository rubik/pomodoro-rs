use futures::future::{abortable, AbortHandle};
use pomolib::state::{PomodoroState, TransitionResult};
use std::sync::{Arc, Mutex};
use tokio::time;

use crate::notifications::Notifier;

pub struct PomodoroClock {
    state: Arc<Mutex<PomodoroState>>,
    abort_handle: Option<AbortHandle>,
    notifier: Arc<Option<Notifier>>,
}

impl PomodoroClock {
    pub fn new(
        state: Arc<Mutex<PomodoroState>>,
        notifier: Arc<Option<Notifier>>,
    ) -> Self {
        Self {
            state,
            notifier,
            abort_handle: None,
        }
    }

    pub fn start(&mut self, s: u64) {
        let state = Arc::clone(&self.state);
        let notifier = Arc::clone(&self.notifier);
        let (task, abort_handle) = abortable(async move {
            let mut wait_sec = s;
            loop {
                time::delay_for(time::Duration::from_secs(wait_sec)).await;
                let mut state = state.lock().unwrap();
                let transition = state.transition();
                if let Some(notify) = &*notifier {
                    notify(&state.phase);
                }
                match transition {
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
