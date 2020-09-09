use structopt::StructOpt;
use tonic::{transport::Server, Request, Response, Status};

use pomodoro::session_server::{Session, SessionServer};
use pomodoro::{GetStateRequest, GetStateResponse, StartRequest, StartResponse, StopRequest, StopResponse, get_state_response::State};

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
    #[structopt(short = "d", long = "db-name", default_value = "pomodoro.sqlite")]
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
    conf: Config,
}

impl PomodoroService {
    pub fn new(conf: Config) -> Self {
        Self { conf }
    }
}

#[tonic::async_trait]
impl Session for PomodoroService {
    async fn start(
        &self,
        request: Request<StartRequest>,
    ) -> Result<Response<StartResponse>, Status> {
        Ok(Response::new(StartResponse::default()))
    }
    async fn stop(
        &self,
        request: Request<StopRequest>,
    ) -> Result<Response<StopResponse>, Status> {
        Ok(Response::new(StopResponse::default()))
    }
    async fn get_state(
        &self,
        request: Request<GetStateRequest>,
    ) -> Result<Response<GetStateResponse>, Status> {
        let state = GetStateResponse {
            state: State::Stopped.into(),
            time_remaining: 234.3,
        };
        Ok(Response::new(state))
    }
}
