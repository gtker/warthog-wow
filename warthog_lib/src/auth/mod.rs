mod logon;
mod reconnect;
mod transfer;

use crate::auth::logon::logon;
use crate::{CredentialProvider, GameFileProvider, KeyStorage, PatchProvider, RealmListProvider};
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use wow_login_messages::all::CMD_AUTH_LOGON_CHALLENGE_Client;
use wow_login_messages::helper::{
    tokio_expect_client_message_protocol, tokio_read_initial_message, InitialMessage,
};
use wow_login_messages::version_2::CMD_REALM_LIST_Client;
use wow_login_messages::version_8::CMD_REALM_LIST_Server;
use wow_login_messages::CollectiveMessage;

pub(crate) async fn auth_server(
    provider: impl CredentialProvider,
    storage: impl KeyStorage,
    patch_provider: impl PatchProvider,
    game_file_provider: impl GameFileProvider,
    realm_list_provider: impl RealmListProvider,
    address: SocketAddr,
) -> io::Result<()> {
    let listener = TcpListener::bind(address).await?;

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let provider = provider.clone();
            let storage = storage.clone();
            let patch_provider = patch_provider.clone();
            let game_file_provider = game_file_provider.clone();
            let realm_list_provider = realm_list_provider.clone();

            tokio::spawn(async move {
                if let Err(a) = auth(
                    stream,
                    provider,
                    storage,
                    patch_provider,
                    game_file_provider,
                    realm_list_provider,
                )
                .await
                {
                    println!("{a}");
                }
            });
        }
    }
}

async fn auth(
    mut stream: TcpStream,
    provider: impl CredentialProvider,
    storage: impl KeyStorage,
    mut patch_provider: impl PatchProvider,
    game_file_provider: impl GameFileProvider,
    realm_list_provider: impl RealmListProvider,
) -> anyhow::Result<()> {
    let c = tokio_read_initial_message(&mut stream).await?;

    match c {
        InitialMessage::Logon(c) => {
            if let Some(data) = patch_provider.get_patch(&c) {
                transfer::transfer(provider, storage, stream, c, data).await?
            } else {
                logon(
                    provider,
                    storage,
                    game_file_provider,
                    realm_list_provider,
                    stream,
                    c,
                )
                .await?
            }
        }
        InitialMessage::Reconnect(c) => {
            reconnect::reconnect(storage, realm_list_provider, stream, c).await?
        }
    }

    Ok(())
}

pub(crate) async fn send_realm_list(
    mut stream: &mut TcpStream,
    c: &CMD_AUTH_LOGON_CHALLENGE_Client,
    mut realm_list_provider: impl RealmListProvider,
) -> Result<(), anyhow::Error> {
    while tokio_expect_client_message_protocol::<CMD_REALM_LIST_Client, _>(
        &mut stream,
        c.protocol_version,
    )
    .await
    .is_ok()
    {
        let realms = realm_list_provider.get_realm_list(c);

        CMD_REALM_LIST_Server { realms }
            .tokio_write_protocol(&mut stream, c.protocol_version)
            .await?;
    }

    Ok(())
}