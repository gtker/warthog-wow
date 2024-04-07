use crate::realm_list::RealmListImpl;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, trace};
use warthog_lib::KeyStorage;
use warthog_messages::{ClientOpcodes, MessageError, ServerOpcodes};

#[tracing::instrument(skip(users, realm))]
pub(crate) async fn start_reply_server(
    users: impl KeyStorage,
    realm: RealmListImpl,
    reply_address: SocketAddr,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(reply_address).await?;
    info!("reply server started");

    loop {
        let (stream, _) = listener.accept().await?;

        let users = users.clone();
        let mut realm = realm.clone();
        tokio::spawn(async move {
            let mut realm_id = None;

            let peer_address = stream.peer_addr();
            match handle_reply(stream, users, realm.clone(), &mut realm_id).await {
                Ok(_) => {}
                Err(_) => {
                    info!(?peer_address, realm_id, "lost connection")
                }
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
                    session_key_request(&mut stream, &mut users, name).await?;
                }
                ServerOpcodes::RegisterRealm { name, address, .. } => {
                    register_realm_request(&mut stream, &mut realm, realm_id, name, address)
                        .await?;
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}

#[tracing::instrument]
async fn session_key_request(
    mut stream: &mut TcpStream,
    users: &mut impl KeyStorage,
    name: String,
) -> Result<(), MessageError> {
    trace!("got session key request");
    let session_key = users
        .get_key_for_user(&name)
        .await
        .map(|a| *a.session_key());

    trace!(?session_key, "looked up key");

    ClientOpcodes::SessionKeyAnswer { name, session_key }
        .tokio_write(&mut stream)
        .await?;
    Ok(())
}

#[tracing::instrument]
async fn register_realm_request(
    mut stream: &mut TcpStream,
    realm: &mut RealmListImpl,
    realm_id: &mut Option<u8>,
    name: String,
    address: String,
) -> Result<(), MessageError> {
    trace!(name, address, "got register realm");
    *realm_id = realm.add_realm(name, address);

    ClientOpcodes::RegisterRealmReply {
        realm_id: *realm_id,
    }
    .tokio_write(&mut stream)
    .await?;
    Ok(())
}
