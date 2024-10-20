use std::{collections::HashSet, net::SocketAddr};

use faux_core::{
    error::FauxError::{ServerError, ServerStartError},
    frame::Codec,
    protocol::{response::Response, FauxPacket},
};
use futures::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::{codec::Decoder, compat::TokioAsyncReadCompatExt};
use yamux::{Config, Connection, Mode};

use crate::client::Client;

pub struct Server {
    client_set: HashSet<Client>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            client_set: HashSet::new(),
        }
    }

    pub async fn start_server(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", 12288)).await?;
        info!("listening on {}", listener.local_addr()?);

        let config = Config::default();

        loop {
            let (stream, addr) = listener.accept().await.map_err(ServerError)?;
            trace!("{}: opened connection", addr);

            tokio::spawn(handle_stream(stream, addr, config.clone()));
        }
    }
}

async fn handle_stream(stream: TcpStream, addr: SocketAddr, config: Config) {
    let mut src = Codec::new().framed(stream);
    let handshake = match src.next().await {
        Some(Ok(FauxPacket::Handshake(handshake))) => handshake,
        err => panic!("{:?}", err),
    };

    debug!(
        "{}: requested a {:?} server forwarding to port {}",
        addr, handshake.protocol, handshake.host_port
    );

    let listener = match TcpListener::bind(format!("0.0.0.0:{}", handshake.preferred_port))
        .await
        .map_err(ServerStartError) {
			Ok(listener) => listener,
			Err(_) => TcpListener::bind("0.0.0.0:0")
				.await
				.map_err(ServerStartError)
				.unwrap()
		};

    src.send(FauxPacket::Response(Response {
        port: listener.local_addr().unwrap().port(),
    }))
    .await
    .unwrap();

    println!("{}", listener.local_addr().unwrap());

    let client = Client::new(handshake);
    let conn = Connection::new(src.into_inner().compat(), config, Mode::Client);

    if let Err(err) = client.start(listener, conn).await {
        println!("{:?}", err);
    }
}
