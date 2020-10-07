use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use tonic::{transport::Server, Request, Response, Status};

use pomodoro::session_server::{Session, SessionServer};
use pomodoro::{
    get_state_response::{Phase, Remaining},
    GetStateRequest, GetStateResponse, StartRequest, StartResponse,
    StopRequest, StopResponse,
};
use pomolib::state::{
    PomodoroPhase, PomodoroSession, PomodoroState, RemainingPeriods,
    ONE_MINUTE,
};

use crate::clock::PomodoroClock;
use crate::notifications::{inotify, Notifier};

pub mod pomodoro {
    tonic::include_proto!("pomodoro");
}

#[derive(StructOpt, Debug)]
#[structopt(name = "pomod", about = "a lightweight pomodoro timer server")]
pub struct Config {
    #[structopt(short = "h", long = "host", default_value = "[::1]")]
    host: String,
    #[structopt(short = "p", long = "port", default_value = "20799")]
    port: u16,
    #[structopt(long = "disable-notifications")]
    disable_notifications: bool,
    #[structopt(
        short = "d",
        long = "db-name",
        default_value = "pomodoro.sqlite"
    )]
    db_name: String,
}

pub async fn run(conf: Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", conf.host, conf.port).parse().unwrap();
    let pomod = PomodoroService::new(conf);

    println!("pomod listening at {}", addr);

    Server::builder()
        .add_service(SessionServer::new(pomod))
        .serve(addr)
        .await?;

    Ok(())
}

pub struct PomodoroService {
    _conf: Config,
    state: Arc<Mutex<PomodoroState>>,
    clock: Arc<Mutex<PomodoroClock>>,
}

impl PomodoroService {
    pub fn new(conf: Config) -> Self {
        let notifier = match conf.disable_notifications {
            true => None,
            false => Some(inotify as Notifier),
        };
        let state = Arc::new(Mutex::new(PomodoroState::default()));
        let clock = Arc::new(Mutex::new(PomodoroClock::new(
            state.clone(),
            Arc::new(notifier),
        )));
        Self {
            _conf: conf,
            state,
            clock,
        }
    }
}

#[tonic::async_trait]
impl Session for PomodoroService {
    async fn start(
        &self,
        request: Request<StartRequest>,
    ) -> Result<Response<StartResponse>, Status> {
        if self.state.lock().unwrap().phase == PomodoroPhase::Working {
            return Err(Status::already_exists(
                "a pomodoro is already in progress",
            ));
        }
        let params = request.get_ref();
        let session = PomodoroSession {
            periods: match params.periods {
                0 => RemainingPeriods::Unlimited,
                n => RemainingPeriods::N(n),
            },
            work_len: match params.work_time {
                0 => 25 * ONE_MINUTE,
                t => t * ONE_MINUTE,
            },
            short_break_len: match params.short_break_time {
                0 => 4 * ONE_MINUTE,
                t => t * ONE_MINUTE,
            },
            long_break_len: match params.long_break_time {
                0 => 20 * ONE_MINUTE,
                t => t * ONE_MINUTE,
            },
            short_breaks_before_long: match params.short_breaks_before_long {
                0 => 3,
                n => n,
            },
        };
        self.clock.lock().unwrap().start(session.work_len.into());
        self.state.lock().unwrap().start(session);
        Ok(Response::new(StartResponse::default()))
    }
    async fn stop(
        &self,
        _request: Request<StopRequest>,
    ) -> Result<Response<StopResponse>, Status> {
        self.clock.lock().unwrap().stop();
        self.state.lock().unwrap().stop();
        Ok(Response::new(StopResponse::default()))
    }
    async fn get_state(
        &self,
        _request: Request<GetStateRequest>,
    ) -> Result<Response<GetStateResponse>, Status> {
        let state = self.state.lock().unwrap();
        let state_response = GetStateResponse {
            phase: phase_to_i32(state.phase),
            time_remaining: state.get_time_remaining().unwrap_or(0),
            remaining_periods: rem_to_i32(state.params.periods.clone()),
            periods: state.params.periods.clone().unwrap_or(0),
        };
        Ok(Response::new(state_response))
    }
}

fn phase_to_i32(phase: PomodoroPhase) -> i32 {
    match phase {
        PomodoroPhase::Stopped => Phase::Stopped,
        PomodoroPhase::Working => Phase::Working,
        PomodoroPhase::ShortBreak => Phase::ShortBreak,
        PomodoroPhase::LongBreak => Phase::LongBreak,
    }
    .into()
}

fn rem_to_i32(rem: RemainingPeriods) -> i32 {
    match rem {
        RemainingPeriods::Unlimited => Remaining::Unlimited,
        RemainingPeriods::N(_) => Remaining::Limited,
    }
    .into()
}
