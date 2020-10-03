extern crate futures;
extern crate tokio;
extern crate tokio_uds_windows;

extern crate tempfile;

use tokio_uds_windows::{UnixStream, UnixListener};
use tempfile::Builder;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::Duration;
use futures::{SinkExt, StreamExt};
use std::io::Read;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use bytes::Bytes;

#[tokio::test]
async fn echo() -> std::io::Result<()> {
    let dir = Builder::new().prefix("tokio-uds-tests").tempdir().unwrap();
    let sock_path = dir.path().join("connect.sock");

    let mut server = UnixListener::bind(&sock_path)?;
    let (mut tx, mut rx) = tokio::sync::mpsc::channel(2);
    std::thread::spawn(move || {
        let mut iter = std::io::stdin().bytes();
        while let Some(key) = iter.next() {
            let key = key.unwrap();
            tx.try_send(key).unwrap();
            println!("Sent {}", key);
        }
    });

    let message = b"Hello from server!";
    //let server_framed = tokio_util::codec::Framed::new(server, tokio_util::codec::LengthDelimitedCodec::new());
    //let (server_tx, server_rx) = server_framed.split();
    tokio::task::spawn(async move {
        while let Some((mut stream, addr)) = server.next().await {
            println!("New conn from: {:?}", &addr);
            /*
            println!("Will send: {:?}", message);
            stream.write_all(message).await.unwrap();
            //stream.flush().await.unwrap();
            let mut buf = vec![0; message.len()];
            while let Ok(len) = stream.read_exact(&mut buf[..message.len()]).await {
                println!("Read message: {}", String::from_utf8(buf.clone()).unwrap());
            }*/
            let client_framed = tokio_util::codec::Framed::new(stream, tokio_util::codec::LengthDelimitedCodec::new());
            let (client_framed_tx, mut client_framed_rx) = client_framed.split();
            while let Some(packet) = client_framed_rx.next().await {
                let packet = packet.unwrap();
                println!("Received packet! {:?} where [0] = {}", &packet, packet[0]);
            }
        }
    });



    tokio::task::spawn(async move {
        let mut client = UnixStream::connect(&sock_path).unwrap().await.unwrap();
        let (mut client_tx, client_rx) = Framed::new(client, LengthDelimitedCodec::new()).split();

        while let Some(key) = rx.next().await {
            println!("Recv key; client will send now");
            let packet_to_send = vec![key];
            client_tx.send(Bytes::from(packet_to_send)).await.unwrap();
        }

        /*
        let mut buf = vec![0; 18];
        let len = client.read_exact(&mut buf[..message.len()]).await.unwrap();
        println!("First byte: {}", buf[0]);
        client.write_all(b"Hello from client!").await.unwrap();
        println!("Received message w {} len", len);
        println!("Message from server: {}", String::from_utf8(buf).unwrap());
        tokio::time::delay_for(Duration::from_millis(1000)).await;
         */
    }).await;


    Ok(())
}
