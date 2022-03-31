use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use futures::FutureExt;
use std::error::Error;

use clap::{Arg, Command};

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

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}
