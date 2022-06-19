use arti_client::{TorClient, TorClientConfig};
use clap::ArgMatches;

pub async fn socks_proxy(args: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let port = args.value_of("listen_port").unwrap();
    arti(port.parse::<u16>().unwrap()).await
}

async fn arti(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let config = TorClientConfig::default();
    let tor_client = TorClient::create_bootstrapped(config).await.unwrap();
    let runtime = tor_client.runtime();
    arti::socks::run_socks_proxy(runtime.clone(), tor_client.clone(), port)
        .await
        .unwrap();
    Ok(())
}
