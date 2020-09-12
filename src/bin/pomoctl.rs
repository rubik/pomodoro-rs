use std::process;
use structopt::StructOpt;

use pomodoro_rs::client::{run, Config};

#[tokio::main(core_threads = 1, max_threads = 1)]
async fn main() {
    let config = Config::from_args();

    if let Err(e) = run(config).await {
        eprintln!("err: {}", e);
        process::exit(1);
    }
}
