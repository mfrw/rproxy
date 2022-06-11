use std::error::Error;

use clap::{Arg, Command};
use futures::FutureExt;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    console_subscriber::init();
    let matches = Command::new("rproxy")
        .version("0.1")
        .author("Muhammad Falak R Wani <falakreyaz@gmail.com>")
        .arg_required_else_help(true)
        .arg(Arg::new("listen_address").short('l').takes_value(true))
        .arg(Arg::new("server_address").short('s').takes_value(true))
        .get_matches();
    let listen_addr = matches.value_of("listen_address").unwrap();
    let server_addr = matches.value_of("server_address").unwrap();

    println!("Listening on: {}", listen_addr);
    println!("Proxying to: {}", server_addr);

    let listener = TcpListener::bind(listen_addr).await?;

    while let Ok((inbound, _)) = listener.accept().await {
        let transfer = transfer(inbound, server_addr.to_string().clone()).map(|r| {
            if let Err(e) = r {
                println!("Failed to transfer; error={}", e);
            }
        });
        tokio::spawn(transfer);
    }
    Ok(())
}

async fn transfer(mut inbound: TcpStream, proxy_addr: String) -> Result<(), Box<dyn Error>> {
    let mut outbound = TcpStream::connect(proxy_addr).await?;
    tokio::io::copy_bidirectional(&mut outbound, &mut inbound).await?;
    Ok(())
}
