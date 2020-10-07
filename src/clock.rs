use futures::future::{abortable, AbortHandle};
use pomolib::state::{PomodoroState, TransitionResult};
use std::sync::{Arc, Mutex};
use tokio::time;

use notify_rust::{Notification, Timeout};
use pomolib::state::PomodoroPhase;

pub struct PomodoroClock {
    state: Arc<Mutex<PomodoroState>>,
    abort_handle: Option<AbortHandle>,
    disable_notifications: bool,
}

impl PomodoroClock {
    pub fn new(
        state: Arc<Mutex<PomodoroState>>,
        disable_notifications: bool,
    ) -> Self {
        Self {
            state,
            disable_notifications,
            abort_handle: None,
        }
    }

    pub fn start(&mut self, s: u64) {
        let state = self.state.clone();
        let (task, abort_handle) = abortable(async move {
            let mut wait_sec = s;
            loop {
                time::delay_for(time::Duration::from_secs(wait_sec)).await;
                let mut state = state.lock().unwrap();
                let transition = state.transition();
                //if !self.disable_notifications {
                    // XXX: It's ugly to put this here, but I tried a lot of
                    // variants with dependency injection and traits and I wasn't
                    // able to make it work with tokio::spawn below.
                    // Traits would also make it easier to test this.
                    let message = match state.phase {
                        PomodoroPhase::Stopped => "Session's over! Go rest!",
                        PomodoroPhase::Working => "Back to work!",
                        PomodoroPhase::ShortBreak => {
                            "Short break started, stand up and stretch!"
                        }
                        PomodoroPhase::LongBreak => {
                            "Long break started, have some rest!"
                        }
                    };
                    let _ = Notification::new()
                        .summary("Pomodoro Timer")
                        .body(message)
                        .timeout(Timeout::Milliseconds(4000))
                        .show();
                //}
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
