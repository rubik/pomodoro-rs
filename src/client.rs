use pomodoro::session_client::SessionClient;
use pomodoro::{
    get_state_response::{Phase, Remaining},
    GetStateRequest, GetStateResponse, StartRequest, StartResponse,
    StopRequest, StopResponse,
};
use structopt::StructOpt;

pub mod pomodoro {
    tonic::include_proto!("pomodoro");
}
use tonic::{transport::Channel, Request, Response};

#[derive(StructOpt)]
#[structopt(name = "pomoctl", about = "a lightweight pomodoro timer client")]
pub struct Config {
    #[structopt(short = "h", long = "host", default_value = "[::1]")]
    host: String,
    #[structopt(short = "p", long = "port", default_value = "20799")]
    port: u16,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    State,
}

pub async fn run(conf: Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", conf.host, conf.port);
    let mut client = SessionClient::connect(addr).await?;

    match conf.cmd {
        Command::State => {
            let state_response = get_state(conf, &mut client).await?;
            print_state(state_response.into_inner());
        }
    };

    Ok(())
}

async fn get_state(
    _conf: Config,
    client: &mut SessionClient<Channel>,
) -> Result<Response<GetStateResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(GetStateRequest {});
    Ok(client.get_state(request).await?)
}

fn print_state(response: GetStateResponse) {
    println!("{:#?}", response);
    match Phase::from_i32(response.phase) {
        Some(Phase::Stopped) => println!("stopped"),
        Some(Phase::Working) => println!("working"),
        Some(Phase::ShortBreak) => println!("short break"),
        Some(Phase::LongBreak) => println!("long break"),
        None => {}
    }
}
