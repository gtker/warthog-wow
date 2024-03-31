use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use warthog_lib::KeyStorage;
use warthog_messages::{ClientOpcodes, MessageError, ServerOpcodes};

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

async fn handle_reply(
    mut stream: TcpStream,
    mut users: impl KeyStorage,
) -> Result<(), MessageError> {
    loop {
        match ServerOpcodes::tokio_read(&mut stream).await {
            Ok(message) => match message {
                ServerOpcodes::RequestSessionKey { name } => {
                    let session_key = users
                        .get_key_for_user(&name)
                        .await
                        .map(|a| *a.session_key());

                    ClientOpcodes::SessionKeyAnswer { name, session_key }
                        .tokio_write(&mut stream)
                        .await?;
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}
