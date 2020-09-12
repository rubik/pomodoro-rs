use structopt::StructOpt;
use tonic::{transport::Channel, Request, Response};

use pomodoro::session_client::SessionClient;
use pomodoro::{
    get_state_response::Phase, GetStateRequest, GetStateResponse,
    StartRequest, StartResponse, StopRequest, StopResponse,
};

pub mod pomodoro {
    tonic::include_proto!("pomodoro");
}

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
    Start {
        #[structopt(short = "p", long = "periods", default_value = "0")]
        periods: u32,
        #[structopt(short = "w", long = "work-time", default_value = "25")]
        work_time: u32,
        #[structopt(short = "s", long = "short-time", default_value = "4")]
        short_break_time: u32,
        #[structopt(short = "l", long = "long-time", default_value = "20")]
        long_break_time: u32,
        #[structopt(long = "short-breaks", default_value = "4")]
        short_breaks_before_long: u32,
    },
    State,
    Stop,
}

struct StartParams {
    periods: u32,
    work_time: u32,
    short_break_time: u32,
    long_break_time: u32,
    short_breaks_before_long: u32,
}

pub async fn run(conf: Config) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("http://{}:{}", conf.host, conf.port);
    let mut client = SessionClient::connect(addr).await?;

    match conf.cmd {
        Command::Start {
            periods,
            work_time,
            short_break_time,
            long_break_time,
            short_breaks_before_long,
        } => {
            let params = StartParams {
                periods,
                work_time,
                short_break_time,
                long_break_time,
                short_breaks_before_long,
            };
            start(&mut client, params).await?;
        }
        Command::State => {
            let state_response = get_state(&mut client).await?;
            print_state(state_response.into_inner());
        }
        Command::Stop => {
            stop(&mut client).await?;
        }
    };

    Ok(())
}

async fn start(
    client: &mut SessionClient<Channel>,
    params: StartParams,
) -> Result<Response<StartResponse>, Box<dyn std::error::Error>> {
    let request = Request::new(StartRequest {
        periods: params.periods,
        work_time: params.work_time,
        short_break_time: params.short_break_time,
        long_break_time: params.long_break_time,
        short_breaks_before_long: params.short_breaks_before_long,
    });
    Ok(client.start(request).await?)
}

async fn get_state(
    client: &mut SessionClient<Channel>,
) -> Result<Response<GetStateResponse>, Box<dyn std::error::Error>> {
    let request = Request::new(GetStateRequest {});
    Ok(client.get_state(request).await?)
}

async fn stop(
    client: &mut SessionClient<Channel>,
) -> Result<Response<StopResponse>, Box<dyn std::error::Error>> {
    let request = Request::new(StopRequest {});
    Ok(client.stop(request).await?)
}

fn print_state(response: GetStateResponse) {
    let (state, remaining) = match Phase::from_i32(response.phase) {
        Some(Phase::Stopped) => ("stopped", None),
        Some(Phase::Working) => ("working", Some(response.time_remaining)),
        Some(Phase::ShortBreak) => ("short-break", Some(response.time_remaining)),
        Some(Phase::LongBreak) => ("long-break", Some(response.time_remaining)),
        None => ("".into(), None),
    };
    let remaining = remaining
        .map(readable_remaining)
        .unwrap_or_else(String::new);
    println!("{} {}", state, remaining);
}

fn readable_remaining(time: u64) -> String {
    if time == 0 {
        return "".into();
    }
    format!("{:02}:{:02}", time / 60, time % 60)
}
