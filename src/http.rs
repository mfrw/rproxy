use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;

use clap::ArgMatches;
use hyper::{
    http,
    service::{make_service_fn, service_fn},
    upgrade::Upgraded,
    Body, Client, Method, Request, Response, Server,
};
use tokio::net::TcpStream;

type HttpClient = Client<hyper::client::HttpConnector>;

pub async fn http_proxy(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let listen_addr = args.value_of("listen_address").unwrap();

    let client = Client::builder()
        .http1_title_case_headers(true)
        .http1_preserve_header_case(true)
        .http2_keep_alive_while_idle(true)
        .http2_adaptive_window(true)
        .build_http();

    let make_service = make_service_fn(move |_| {
        let client = client.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| proxy(client.clone(), req))) }
    });

    let server = Server::bind(&listen_addr.parse::<SocketAddr>()?).serve(make_service);
    println!("Listening on: http://{}", listen_addr);
    if let Err(e) = server.await {
        eprintln!("Server error: {e}");
    }
    Ok(())
}

async fn proxy(client: HttpClient, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    if req.method() == Method::CONNECT {
        if let Some(addr) = host_addr(req.uri()) {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = tunnel(upgraded, addr).await {
                            eprintln!("server io error: {e}");
                        };
                    }
                    Err(e) => eprintln!("upgrade error: {e}"),
                }
            });

            Ok(Response::new(Body::empty()))
        } else {
            eprintln!("CONNECT HOST is not a socket addr: {:?}", req.uri());
            let mut resp = Response::new(Body::from("CONNECT must be a socket address"));
            *resp.status_mut() = http::StatusCode::BAD_REQUEST;
            Ok(resp)
        }
    } else {
        client.request(req).await
    }
}

async fn tunnel(mut upgraded: Upgraded, addr: String) -> std::io::Result<()> {
    let mut server = TcpStream::connect(addr).await?;
    tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;
    Ok(())
}

fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}
