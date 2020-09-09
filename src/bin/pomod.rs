use std::process;
use structopt::StructOpt;
use tokio;

use pomodoro_rs::server::{Config, run};

#[tokio::main]
async fn main() {
    let config = Config::from_args();

    if let Err(e) = run(config).await {
        eprintln!("err: {}", e);
        process::exit(1);
    }
}
