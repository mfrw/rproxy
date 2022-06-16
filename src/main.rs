use std::error::Error;

use clap::{Arg, Command};

mod tcp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    console_subscriber::init();
    let matches = Command::new("rproxy")
        .version("0.1")
        .author("Muhammad Falak R Wani <falakreyaz@gmail.com>")
        .arg_required_else_help(true)
        .arg(Arg::new("listen_address").short('l').takes_value(true))
        .subcommand(
            Command::new("tcp").arg(Arg::new("server_address").short('s').takes_value(true)),
        )
        .subcommand(Command::new("http"))
        .get_matches();
    match matches.subcommand() {
        Some(("tcp", a)) => tcp::tcp_proxy(a).await,
        _ => panic!(),
    }
}

