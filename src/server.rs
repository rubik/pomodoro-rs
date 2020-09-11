use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use tonic::{transport::Server, Request, Response, Status};

use crate::state::{
    PomodoroPhase, PomodoroSession, PomodoroState, RemainingPeriods,
    ONE_MINUTE,
};
use crate::clock::PomodoroClock;
use pomodoro::session_server::{Session, SessionServer};
use pomodoro::{
    get_state_response::{Phase, Remaining},
    GetStateRequest, GetStateResponse, StartRequest, StartResponse,
    StopRequest, StopResponse,
};

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

impl From<PomodoroPhase> for i32 {
    fn from(phase: PomodoroPhase) -> i32 {
        match phase {
            PomodoroPhase::Stopped => Phase::Stopped,
            PomodoroPhase::Working => Phase::Working,
            PomodoroPhase::ShortBreak => Phase::ShortBreak,
            PomodoroPhase::LongBreak => Phase::LongBreak,
        }
        .into()
    }
}

impl From<RemainingPeriods> for i32 {
    fn from(phase: RemainingPeriods) -> i32 {
        match phase {
            RemainingPeriods::Unlimited => Remaining::Unlimited,
            RemainingPeriods::N(_) => Remaining::Limited,
        }
        .into()
    }
}

pub struct PomodoroService {
    conf: Config,
    state: Arc<Mutex<PomodoroState>>,
    clock: Option<PomodoroClock>,
}

impl PomodoroService {
    pub fn new(conf: Config) -> Self {
        Self {
            conf,
            state: Arc::new(Mutex::new(PomodoroState::default())),
            clock: None,
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
        self.state.lock().unwrap().start(session);
        self.clock = PomodoroClock::new(self.state.clone());
        // create timed task
        Ok(Response::new(StartResponse::default()))
    }
    async fn stop(
        &self,
        request: Request<StopRequest>,
    ) -> Result<Response<StopResponse>, Status> {
        self.state.lock().unwrap().stop();
        Ok(Response::new(StopResponse::default()))
    }
    async fn get_state(
        &self,
        request: Request<GetStateRequest>,
    ) -> Result<Response<GetStateResponse>, Status> {
        let state = self.state.lock().unwrap();
        let state_response = GetStateResponse {
            phase: state.phase.into(),
            time_remaining: state.get_time_remaining().unwrap_or(0),
            remaining_periods: state.params.periods.clone().into(),
            periods: state.params.periods.clone().unwrap_or(0),
        };
        Ok(Response::new(state_response))
    }
}
