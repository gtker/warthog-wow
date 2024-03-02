mod auth;

use std::future::Future;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::auth::auth;
pub use wow_login_messages::all::CMD_AUTH_LOGON_CHALLENGE_Client;
pub use wow_login_messages::all::Population;
pub use wow_login_messages::version_8::Realm;
pub use wow_login_messages::version_8::RealmCategory;
pub use wow_login_messages::version_8::RealmType;
pub use wow_srp::normalized_string::NormalizedString;
pub use wow_srp::server::SrpServer;
pub use wow_srp::server::SrpVerifier;
pub use wow_srp::PASSWORD_VERIFIER_LENGTH;
pub use wow_srp::SALT_LENGTH;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Options {
    /// Address to host the auth server on.
    pub address: SocketAddr,
    /// Shift around numbers on the PIN grid.
    pub randomize_pin_grid: bool,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Credentials {
    pub password_verifier: [u8; PASSWORD_VERIFIER_LENGTH as usize],
    pub salt: [u8; SALT_LENGTH as usize],
    pub pin: Option<u32>,
}

pub trait CredentialProvider: Clone + Send + Sync + 'static {
    fn get_user(&mut self, username: &str) -> impl Future<Output = Option<Credentials>> + Send;

    fn add_user(
        &mut self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Option<()>> + Send;
}

pub trait KeyStorage: Clone + Send + Sync + 'static {
    fn add_key(&mut self, username: String, server: SrpServer) -> impl Future<Output = ()> + Send;

    fn get_key_for_user(
        &mut self,
        username: &str,
    ) -> impl Future<Output = Option<SrpServer>> + Send;
}

pub trait PatchProvider: Clone + Send + Sync + 'static {
    fn get_patch(
        &mut self,
        message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Vec<u8>>> + Send;
}

pub trait GameFileProvider: Clone + Send + Sync + 'static {
    fn get_game_files(
        &mut self,
        message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Vec<u8>>> + Send;
}

pub trait RealmListProvider: Clone + Send + Sync + 'static {
    fn get_realm_list(
        &mut self,
        message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Vec<Realm>> + Send;
}

pub async fn start_auth_server(
    provider: impl CredentialProvider,
    storage: impl KeyStorage,
    patch_provider: impl PatchProvider,
    game_file_provider: impl GameFileProvider,
    realm_list_provider: impl RealmListProvider,
    options: Options,
) -> anyhow::Result<()> {
    let options: &'static mut _ = Box::leak(Box::new(options));
    let listener = TcpListener::bind(options.address).await?;

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let provider = provider.clone();
            let storage = storage.clone();
            let patch_provider = patch_provider.clone();
            let game_file_provider = game_file_provider.clone();
            let realm_list_provider = realm_list_provider.clone();
            let options: &'static _ = &*options;

            tokio::spawn(async move {
                if let Err(a) = auth(
                    stream,
                    provider,
                    storage,
                    patch_provider,
                    game_file_provider,
                    realm_list_provider,
                    options,
                )
                .await
                {
                    println!("{a}");
                }
            });
        }
    }
}
