mod auth;

use std::future::Future;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

use crate::auth::{auth, InternalError};

pub use wow_login_messages::all::CMD_AUTH_LOGON_CHALLENGE_Client;
pub use wow_login_messages::all::CMD_AUTH_RECONNECT_CHALLENGE_Client;
pub use wow_login_messages::all::Population;
pub use wow_login_messages::errors::ExpectedOpcodeError;
pub use wow_login_messages::version_8::opcodes::ClientOpcodeMessage;
pub use wow_login_messages::version_8::Realm;
pub use wow_login_messages::version_8::RealmCategory;
pub use wow_login_messages::version_8::RealmType;
pub use wow_login_messages::version_8::Realm_RealmFlag;
pub use wow_login_messages::version_8::Realm_RealmFlag_SpecifyBuild;
pub use wow_srp::error::InvalidPublicKeyError;
pub use wow_srp::matrix_card::MatrixCard;
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
    /// Maximum amount of concurrent users.
    pub max_concurrent_users: u32,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Credentials {
    pub password_verifier: [u8; PASSWORD_VERIFIER_LENGTH as usize],
    pub salt: [u8; SALT_LENGTH as usize],
    pub pin: Option<u32>,
    pub matrix_card: Option<MatrixCardOptions>,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct MatrixCardOptions {
    pub matrix_card: MatrixCard,
    pub challenge_count: u8,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ExpectedOpcode {
    LoginOrReconnect,
    LogonProof,
    ReconnectProof,
    XferOrResume,
}

pub trait CredentialProvider: Clone + Send + Sync + 'static {
    fn get_user(
        &mut self,
        username: &str,
        message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Credentials>> + Send;

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
    ) -> impl Future<Output = Option<Arc<[u8]>>> + Send;
}

pub trait GameFileProvider: Clone + Send + Sync + 'static {
    fn get_game_files(
        &mut self,
        message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Arc<[u8]>>> + Send;
}

pub trait RealmListProvider: Clone + Send + Sync + 'static {
    fn get_realm_list(
        &mut self,
        message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Vec<Realm>> + Send;
}

pub trait ErrorProvider: Clone + Send + Sync + 'static {
    fn message_invalid(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        opcode: ClientOpcodeMessage,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;
    fn username_invalid(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn username_not_found(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_password(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_integrity_check(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_pin(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn pin_not_sent(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn matrix_card_not_sent(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_matrix_card(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_user_attempted_reconnect(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_reconnect_integrity_check(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_reconnect_proof(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_public_key(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        err: InvalidPublicKeyError,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn io_error(&mut self, io: std::io::Error, addr: SocketAddr)
        -> impl Future<Output = ()> + Send;

    fn provided_file_too_large(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        size: usize,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn transfer_offset_too_large(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        size: u64,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;

    fn invalid_expected_opcode(
        &mut self,
        err: ExpectedOpcodeError,
        expected_opcode: ExpectedOpcode,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send;
}

pub async fn start_auth_server(
    provider: impl CredentialProvider,
    storage: impl KeyStorage,
    patch_provider: impl PatchProvider,
    game_file_provider: impl GameFileProvider,
    realm_list_provider: impl RealmListProvider,
    error_provider: impl ErrorProvider,
    should_run: Arc<AtomicBool>,
    options: Options,
) -> std::io::Result<()> {
    let options: &'static mut _ = Box::leak(Box::new(options));
    let listener = TcpListener::bind(options.address).await?;

    let concurrent_connections = Arc::new(AtomicU32::new(0));

    let should_run = tokio::spawn(async move {
        while should_run.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::new(0, 100)).await;
        }
    });

    let main_loop = tokio::spawn(async move {
        loop {
            let connections = concurrent_connections.clone();
            if connections.load(Ordering::SeqCst) > options.max_concurrent_users {
                continue;
            }

            if let Ok((stream, addr)) = listener.accept().await {
                connections.fetch_add(1, Ordering::SeqCst);
                let provider = provider.clone();
                let storage = storage.clone();
                let patch_provider = patch_provider.clone();
                let game_file_provider = game_file_provider.clone();
                let realm_list_provider = realm_list_provider.clone();
                let error_provider = error_provider.clone();
                let options: &'static _ = &*options;

                tokio::spawn(async move {
                    if let Err(err) = auth(
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
                        dispatch_error(error_provider, err, addr).await
                    }

                    connections.fetch_sub(1, Ordering::SeqCst);
                });
            }
        }
    });

    tokio::select! {
        _ = should_run => {}
        _ = main_loop => {}
    }

    Ok(())
}

async fn dispatch_error(
    mut error_provider: impl ErrorProvider,
    a: InternalError,
    addr: SocketAddr,
) {
    match a {
        InternalError::MessageInvalid { opcode, message } => {
            error_provider.message_invalid(message, opcode, addr).await
        }
        InternalError::UsernameInvalid { message } => {
            error_provider.username_invalid(message, addr).await
        }
        InternalError::UsernameNotFound { message } => {
            error_provider.username_not_found(message, addr).await
        }
        InternalError::InvalidPasswordForUser { message } => {
            error_provider.invalid_password(message, addr).await;
        }
        InternalError::InvalidIntegrityCheckForUser { message } => {
            error_provider.invalid_integrity_check(message, addr).await
        }
        InternalError::PinInvalidForUser { message } => {
            error_provider.invalid_pin(message, addr).await
        }
        InternalError::PinNotSentForUser { message } => {
            error_provider.pin_not_sent(message, addr).await
        }
        InternalError::MatrixCardDataNotSentForUser { message } => {
            error_provider.matrix_card_not_sent(message, addr).await
        }
        InternalError::MatrixCardInvalidForUser { message } => {
            error_provider.invalid_matrix_card(message, addr).await
        }
        InternalError::InvalidUserAttemptedReconnect { message } => {
            error_provider
                .invalid_user_attempted_reconnect(message, addr)
                .await
        }
        InternalError::InvalidReconnectIntegrityCheckForUser { message } => {
            error_provider
                .invalid_reconnect_integrity_check(message, addr)
                .await
        }
        InternalError::InvalidReconnectProofForUser { message } => {
            error_provider.invalid_reconnect_proof(message, addr).await
        }
        InternalError::ExpectedOpcodeError { expected, err } => {
            error_provider
                .invalid_expected_opcode(err, expected, addr)
                .await
        }
        InternalError::InvalidPublicKey { message, err } => {
            error_provider.invalid_public_key(message, err, addr).await
        }
        InternalError::Io { err } => error_provider.io_error(err, addr).await,
        InternalError::ProvidedFileTooLarge { message, size } => {
            error_provider
                .provided_file_too_large(message, size, addr)
                .await
        }
        InternalError::TransferOffsetTooLarge { message, size } => {
            error_provider
                .transfer_offset_too_large(message, size, addr)
                .await
        }
    }
}
