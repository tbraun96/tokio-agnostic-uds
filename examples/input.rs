extern crate futures;
extern crate tokio;
extern crate tokio_uds_windows;

extern crate tempfile;

use tokio_uds_windows::*;
use tempfile::Builder;
use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncBufReadExt};
use tokio::time::Duration;
use futures::{SinkExt, StreamExt};
use std::io::Read;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use bytes::Bytes;
use std::io::BufRead;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let dir = Builder::new().prefix("tokio-uds-tests").tempdir().unwrap();
    let sock_path = dir.path().join("connect.sock");

    let mut server = UnixListener::bind(&sock_path)?;
    let (mut tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut iter = stdin.lock().lines();
        while let Some(keys) = iter.next() {
            let keys = keys.unwrap();
            println!("Sent {}", &keys);
            tx.send(Bytes::from(keys)).unwrap();
        }
    });

    tokio::task::spawn(async move {
        while let Some((mut stream, addr)) = server.next().await {
            println!("New conn from: {:?}", &addr);
            let client_framed = tokio_util::codec::Framed::new(stream, tokio_util::codec::LengthDelimitedCodec::new());
            let (client_framed_tx, mut client_framed_rx) = client_framed.split();
            while let Some(packet) = client_framed_rx.next().await {
                let packet = packet.unwrap();
                println!("Received packet! {:?}", &packet);
            }
        }
    });



    tokio::task::spawn(async move {
        let mut client = UnixStream::connect(&sock_path).await.unwrap();
        let (mut client_tx, client_rx) = Framed::new(client, LengthDelimitedCodec::new()).split();

        while let Some(keys) = rx.next().await {
            println!("Recv key; client will send now");
            client_tx.send(keys).await.unwrap();
        }
    }).await;


    Ok(())
}
