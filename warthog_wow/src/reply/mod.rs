use crate::realm_list::RealmListImpl;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use warthog_lib::KeyStorage;
use warthog_messages::{ClientOpcodes, MessageError, ServerOpcodes};

pub(crate) async fn start_reply_server(
    users: impl KeyStorage,
    realm: RealmListImpl,
    reply_address: SocketAddr,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(reply_address).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let users = users.clone();
        let mut realm = realm.clone();
        tokio::spawn(async move {
            let peer_address = stream.peer_addr().unwrap();
            let mut realm_id = None;

            match handle_reply(stream, users, realm.clone(), &mut realm_id).await {
                Ok(_) => {}
                Err(_) => println!("[REPLY] Lost connection to {}", peer_address),
            }

            if let Some(realm_id) = realm_id {
                realm.remove_realm(realm_id);
            }
        });
    }
}

async fn handle_reply(
    mut stream: TcpStream,
    mut users: impl KeyStorage,
    mut realm: RealmListImpl,
    realm_id: &mut Option<u8>,
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
                ServerOpcodes::RegisterRealm { name, address, .. } => {
                    *realm_id = realm.add_realm(name, address);

                    ClientOpcodes::RegisterRealmReply {
                        realm_id: *realm_id,
                    }
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
