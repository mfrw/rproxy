use clap::ArgMatches;
use futures::FutureExt;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

pub async fn tcp_proxy(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let listen_addr = args.value_of("listen_address").unwrap();
    let server_addr = args.value_of("server_address").unwrap();

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
