use std::time::SystemTime;

pub const ONE_MINUTE: u32 = 60;

fn seconds_since(t: SystemTime) -> u64 {
    SystemTime::now()
        .duration_since(t)
        .expect("time went backwards")
        .as_secs()
}

#[derive(Copy, Clone, PartialEq)]
pub enum PomodoroPhase {
    Stopped,
    Working,
    ShortBreak,
    LongBreak,
}

pub enum TransitionResult {
    Stopped,
    NextTransitionIn(u32),
}

#[derive(Clone)]
pub enum RemainingPeriods {
    Unlimited,
    N(u32),
}

impl RemainingPeriods {
    pub fn unwrap_or(self, default: u32) -> u32 {
        match self {
            Self::Unlimited => default,
            Self::N(n) => n,
        }
    }

    pub fn decrement(&mut self) {
        if let Self::N(ref mut n) = self {
            *n = *n - 1;
        }
    }

    pub fn done(&self) -> bool {
        match self {
            Self::Unlimited => false,
            Self::N(0) => true,
            Self::N(_) => false,
        }
    }
}

/// Pomodoro session parameters.
pub struct PomodoroSession {
    /// The number of work periods in the session.
    pub periods: RemainingPeriods,
    /// The length, in seconds, of the work period.
    pub work_len: u32,
    /// The length, in seconds, of the short break period.
    pub short_break_len: u32,
    /// The length, in seconds, of the long break period.
    pub long_break_len: u32,
    /// The number of short breaks before a long break.
    pub short_breaks_before_long: u32,
}

impl Default for PomodoroSession {
    fn default() -> Self {
        Self {
            periods: RemainingPeriods::Unlimited,
            work_len: 25 * ONE_MINUTE,
            short_break_len: 4 * ONE_MINUTE,
            long_break_len: 20 * ONE_MINUTE,
            short_breaks_before_long: 3,
        }
    }
}

pub struct PomodoroState {
    /// The phase in which the pomodoro timer is in.
    pub phase: PomodoroPhase,
    /// The total length, in seconds, of the current phase.
    pub current_len: Option<u32>,
    /// The Unix timestamp of the instant in which the current phase was
    /// started.
    pub current_started_at: Option<SystemTime>,
    /// The number of short breaks already done.
    pub short_breaks_done: u32,
    /// Session parameters.
    pub params: PomodoroSession,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self {
            phase: PomodoroPhase::Stopped,
            current_len: None,
            current_started_at: None,
            short_breaks_done: 0,
            params: PomodoroSession::default(),
        }
    }
}

impl PomodoroState {
    pub fn start(&mut self, params: PomodoroSession) {
        self.phase = PomodoroPhase::Working;
        self.current_len = Some(params.work_len);
        self.current_started_at = Some(SystemTime::now());
        self.short_breaks_done = 0;
        self.params = params;
    }

    pub fn transition(&mut self) -> TransitionResult {
        if self.phase == PomodoroPhase::ShortBreak {
            self.short_breaks_done += 1;
        } else if self.phase == PomodoroPhase::Working {
            self.params.periods.decrement();
            if self.params.periods.done() {
                self.stop();
                return TransitionResult::Stopped;
            }
        }
        self.phase = match self.phase {
            PomodoroPhase::Stopped => PomodoroPhase::Stopped,
            PomodoroPhase::ShortBreak => PomodoroPhase::Working,
            PomodoroPhase::LongBreak => PomodoroPhase::Working,
            PomodoroPhase::Working => {
                if self.short_breaks_done
                    == self.params.short_breaks_before_long
                {
                    PomodoroPhase::LongBreak
                } else {
                    PomodoroPhase::ShortBreak
                }
            }
        };
        let s = match self.phase {
            PomodoroPhase::Stopped => {
                self.stop();
                return TransitionResult::Stopped;
            }
            PomodoroPhase::Working => self.params.work_len,
            PomodoroPhase::ShortBreak => self.params.short_break_len,
            PomodoroPhase::LongBreak => self.params.long_break_len,
        };
        TransitionResult::NextTransitionIn(s)
    }

    pub fn stop(&mut self) {
        self.phase = PomodoroPhase::Stopped;
        self.params = PomodoroSession::default();
        self.current_len = None;
        self.current_started_at = None;
        self.short_breaks_done = 0;
    }

    pub fn get_time_remaining(&self) -> Option<u64> {
        self.current_started_at.map(|s| {
            let elapsed = seconds_since(s);
            let phase_time = match self.phase {
                PomodoroPhase::Working => self.params.work_len,
                PomodoroPhase::ShortBreak => self.params.short_break_len,
                PomodoroPhase::LongBreak => self.params.long_break_len,
                PomodoroPhase::Stopped => return 0,
            } as u64;
            // XXX: this can overflow if the phase time is changed, in theory
            phase_time - elapsed
        })
    }
}
