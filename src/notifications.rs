use notify_rust::{Notification, Timeout};
use pomolib::state::PomodoroPhase;

pub type Notifier = fn(&PomodoroPhase);

pub fn inotify(phase: &PomodoroPhase) {
    let message = match *phase {
        PomodoroPhase::Stopped => "Session's over! Go rest!",
        PomodoroPhase::Working => "Back to work!",
        PomodoroPhase::ShortBreak => {
            "Short break started, stand up and stretch!"
        }
        PomodoroPhase::LongBreak => "Long break started, have some rest!",
    };
    let _ = Notification::new()
        .summary("Pomodoro Timer")
        .body(message)
        .timeout(Timeout::Milliseconds(4000))
        .show();
}
