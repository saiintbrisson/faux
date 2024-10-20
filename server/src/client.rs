use std::{collections::HashSet, net::SocketAddr};

use faux_core::{error::Result, protocol::handshake::Handshake};
use futures::{io::copy, AsyncReadExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Sender},
};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

pub struct Client {
    handshake: Handshake,
    connection_set: HashSet<Connection>,
}

impl Client {
    pub fn new(handshake: Handshake) -> Self {
        Self {
            handshake,
            connection_set: Default::default(),
        }
    }

    pub async fn start(
        self,
        listener: TcpListener,
        conn: yamux::Connection<Compat<TcpStream>>,
    ) -> Result<()> {
        let (tx, mut rx) = channel(150);
        tokio::spawn(Client::start_listener(listener, tx));

        let ctrl = conn.control();
        tokio::spawn(yamux::into_stream(conn).for_each(|_| futures::future::ready(())));

        while let Some(msg) = rx.recv().await {
            let mut ctrl = ctrl.clone();

            let _ = match msg {
                Message::NewConnection(Connection(stream, addr)) => tokio::spawn(async move {
                    let (mut c_rx, mut c_tx) = stream.compat().split();
                    let (mut s_rx, mut s_tx) = AsyncReadExt::split(
                        ctrl.open_stream().await.map_err(|err| dbg!(err)).unwrap(),
                    );

                    println!("received connection from {}, forwarding to client", addr);

                    tokio::spawn(async move { copy(&mut c_rx, &mut s_tx).await });
                    tokio::spawn(async move { copy(&mut s_rx, &mut c_tx).await });
                }),
                Message::EndConnection(_) => todo!(),
            };
        }

        Ok(())
    }

    async fn start_listener(listener: TcpListener, tx: Sender<Message>) {
        while let Ok((stream, addr)) = listener.accept().await {
            if let Err(_) = tx
                .send(Message::NewConnection(Connection(stream, addr)))
                .await
            {}
        }
    }
}

enum Message {
    NewConnection(Connection),
    EndConnection(SocketAddr),
}

#[derive(Debug)]
pub struct Connection(TcpStream, SocketAddr);
