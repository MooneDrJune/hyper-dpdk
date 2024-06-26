use std::env;
use std::io::Read;
use bytes::{Buf, Bytes};
use http::Request;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::TcpStream;
use http_body_util::{BodyExt, Empty};

#[path = "../benches/support/mod.rs"]
mod support;
use support::TokioIo;

fn main() {
    pretty_env_logger::init();
    let args: Vec<String> = env::args().collect();
    tokio::fstack_init(args.len(), args);

    let url = String::from("https://api.binance.com/api/v3/depth?symbol=BNBBTC&limit=1000");


    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let url = url.parse::<hyper::Uri>().unwrap();

    rt.block_on(async {
        let local = tokio::task::LocalSet::new();
        local.run_until(async move {
            let host = url.host().expect("uri has no host");
            let port = url.port_u16().unwrap_or(443);
            let addr = format!("{}:{}", host, port);
            println!("addr: {}", addr);

            let stream = TcpStream::connect(addr).await.expect("connect failed");
            let io = TokioIo::new(stream);

            match hyper::client::conn::http1::handshake(io).await {
                Ok((mut client, connection)) => {
                    println!("Handshake successful!");

                    tokio::task::spawn_local(async move {
                        if let Err(e) = connection.await {
                            eprintln!("Connection error: {}", e);
                        }
                    });

                    let req = Request::builder()
                        .method("GET")
                        .uri(url)
                        .body(Empty::<Bytes>::new())
                        .unwrap();

                    match client.send_request(req).await {
                        Ok(mut response) => {
                            println!("Response: {}", response.status());
                            let mut reader = response.collect().await.unwrap().aggregate().reader();
                            let mut body_str = String::new();
                            reader.read_to_string(&mut body_str).unwrap();

                            println!("Body: {}", body_str);
                        }
                        Err(e) => {
                            eprintln!("Request error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("handshake failed: {}", e);
                }
            }
        }).await;
    });
}

