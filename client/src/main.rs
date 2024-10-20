use anyhow::Result;
use bytes::Bytes;
use faux_core::{
    frame::Codec,
    protocol::{handshake::Handshake, BridgeProtocol, FauxPacket},
};
use futures::{io::copy, AsyncReadExt, SinkExt, StreamExt, TryStreamExt};
use tokio::{net::TcpStream, runtime::Builder as RuntimeBuilder};
use tokio_util::{codec::Decoder, compat::TokioAsyncReadCompatExt};
use yamux::{Config, Connection, Mode};

fn main() -> Result<()> {
    let runtime = RuntimeBuilder::new_multi_thread().enable_all().build()?;
    runtime.block_on(a())
}

async fn a() -> Result<()> {
    let stream = TcpStream::connect("132.226.243.45:12288").await.unwrap();
    let mut src = Codec::new().framed(stream);

    let port = std::env::args()
        .nth(1)
        .and_then(|port| port.parse().ok())
        .expect("Expected port");
		
	let preferred = std::env::args()
        .nth(2)
        .and_then(|port| port.parse().ok())
        .unwrap_or(0u16);

    src.send(FauxPacket::Handshake(Handshake {
        version: Bytes::from_static(b"Faux_v1.0.0"),
        os_name: Bytes::from_static(b"Windows 10"),
        protocol: BridgeProtocol::TCP,
        host_port: port,
		preferred_port: preferred
    }))
    .await
    .unwrap();

    let response = match src.next().await {
        Some(Ok(FauxPacket::Response(response))) => response,
        _ => return Ok(()),
    };

    println!("Awaiting connections: {:?}", response.port);

    let conn = Connection::new(src.into_inner().compat(), Config::default(), Mode::Server);

    yamux::into_stream(conn)
        .try_for_each_concurrent(None, |stream| async move {
            let (mut s_rx, mut s_tx) = TcpStream::connect(format!("127.0.0.1:{}", port))
                .await
                .unwrap()
                .compat()
                .split();
            let (mut c_rx, mut c_tx) = AsyncReadExt::split(stream);

            println!("received connection from server, forwarding to app");

            tokio::spawn(async move { copy(&mut c_rx, &mut s_tx).await });
            tokio::spawn(async move { copy(&mut s_rx, &mut c_tx).await });

            Ok(())
        })
        .await
        .unwrap();

    loop {}
}
