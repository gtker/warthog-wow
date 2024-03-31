use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use warthog_lib::KeyStorage;

pub(crate) async fn start_reply_server(
    storage: impl KeyStorage,
    reply_address: SocketAddr,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(reply_address).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let users = storage.clone();
        tokio::spawn(async move {
            let peer_address = stream.peer_addr().unwrap();

            match handle_reply(stream, users).await {
                Ok(_) => {}
                Err(_) => println!("[REPLY] Lost connection to {}", peer_address),
            }
        });
    }
}

async fn handle_reply(mut stream: TcpStream, mut users: impl KeyStorage) -> std::io::Result<()> {
    let mut buf = [0_u8; 1];
    loop {
        stream.read_exact(&mut buf).await?;

        let name_length = buf[0];

        let mut v = Vec::with_capacity(name_length.into());

        for _ in 0..name_length {
            stream.read_exact(&mut buf).await?;
            v.push(buf[0]);
        }

        let name = match String::from_utf8(v.clone()) {
            Ok(s) => s,
            Err(e) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
            }
        };

        let session_key = {
            let u = users.get_key_for_user(&name).await;
            u.map(|a| *a.session_key())
        };

        let vec_size = 2 + name.as_bytes().len() + if session_key.is_some() { 40 } else { 0 };
        let mut buffer = Vec::with_capacity(vec_size);

        buffer.push(name.len() as u8);

        for v in name.as_bytes() {
            buffer.push(*v);
        }

        if let Some(session_key) = session_key {
            buffer.push(1);

            for v in session_key {
                buffer.push(v);
            }
        } else {
            buffer.push(0);
        }

        stream.write_all(&buffer).await?;
    }
}
