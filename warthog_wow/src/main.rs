use clap::Parser;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool};
use std::sync::{Arc, Mutex};
use warthog_lib::{
    start_auth_server, CMD_AUTH_LOGON_CHALLENGE_Client, CMD_AUTH_RECONNECT_CHALLENGE_Client,
    ClientOpcodeMessage, CredentialProvider, Credentials, ErrorProvider, ExpectedOpcode,
    ExpectedOpcodeError, GameFileProvider, InvalidPublicKeyError, KeyStorage, MatrixCard,
    MatrixCardOptions, NormalizedString, Options, PatchProvider, Realm, RealmCategory,
    RealmListProvider, RealmType, SrpServer, SrpVerifier,
};

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
    /// Address to host auth server on.
    #[arg(short, long, default_value = "0.0.0.0:3724")]
    address: SocketAddr,
    /// Randomize PIN grid number locations.
    #[arg(short, long, default_value = "false")]
    randomize_pin_grid: bool,
}

impl Args {
    fn to_options(self) -> Options {
        Options {
            address: self.address,
            randomize_pin_grid: self.randomize_pin_grid,
            max_concurrent_users: 1000,
        }
    }
}

#[derive(Copy, Clone)]
struct ProviderImpl {}

const DIGIT_COUNT: u8 = 2;
const CHALLENGE_COUNT: u8 = 1;
const HEIGHT: u8 = 8;
const WIDTH: u8 = 8;

impl CredentialProvider for ProviderImpl {
    fn get_user(
        &mut self,
        username: &str,
        message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Credentials>> + Send {
        let v = SrpVerifier::from_username_and_password(
            NormalizedString::new(username).unwrap(),
            NormalizedString::new(username).unwrap(),
        );

        let matrix_card = if message.version.supports_matrix_card() {
            Some(MatrixCardOptions {
                matrix_card: MatrixCard::from_data(
                    DIGIT_COUNT,
                    HEIGHT,
                    WIDTH,
                    vec![0; DIGIT_COUNT as usize * HEIGHT as usize * WIDTH as usize],
                )
                .unwrap(),
                challenge_count: CHALLENGE_COUNT,
            })
        } else {
            None
        };

        let pin = if message.version.supports_pin() {
            Some(1234)
        } else {
            None
        };

        async move {
            Some(Credentials {
                password_verifier: *v.password_verifier(),
                salt: *v.salt(),
                pin,
                matrix_card,
            })
        }
    }

    fn add_user(
        &mut self,
        _username: &str,
        _password: &str,
    ) -> impl Future<Output = Option<()>> + Send {
        async move { None }
    }
}

#[derive(Clone)]
struct KeyImpl {
    inner: Arc<Mutex<HashMap<String, SrpServer>>>,
}

impl KeyImpl {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Default::default())),
        }
    }
}

impl KeyStorage for KeyImpl {
    fn add_key(&mut self, username: String, server: SrpServer) -> impl Future<Output = ()> + Send {
        async move {
            self.inner.lock().unwrap().insert(username, server);
        }
    }

    fn get_key_for_user(
        &mut self,
        username: &str,
    ) -> impl Future<Output = Option<SrpServer>> + Send {
        async move { self.inner.lock().unwrap().get(username).cloned() }
    }
}

#[derive(Clone)]
struct PatchImpl {}

impl PatchProvider for PatchImpl {
    fn get_patch(
        &mut self,
        _message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Arc<[u8]>>> + Send {
        async move { None }
    }
}

#[derive(Clone)]
struct GameFileImpl {}

impl GameFileProvider for GameFileImpl {
    fn get_game_files(
        &mut self,
        _message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Arc<[u8]>>> + Send {
        async move { None }
    }
}

#[derive(Clone)]
struct RealmListImpl {}

impl RealmListProvider for RealmListImpl {
    fn get_realm_list(
        &mut self,
        _message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Vec<Realm>> + Send {
        async move {
            vec![
                Realm {
                    realm_type: RealmType::PlayerVsEnvironment,
                    locked: false,
                    flag: Default::default(),
                    name: "Test Realm2".to_string(),
                    address: "localhost:8085".to_string(),
                    population: Default::default(),
                    number_of_characters_on_realm: 2,
                    category: RealmCategory::One,
                    realm_id: 1,
                },
                Realm {
                    realm_type: RealmType::PlayerVsEnvironment,
                    locked: false,
                    flag: Default::default(),
                    name: "Test Realm".to_string(),
                    address: "localhost:8085".to_string(),
                    population: Default::default(),
                    number_of_characters_on_realm: 3,
                    category: RealmCategory::Two,
                    realm_id: 0,
                },
            ]
        }
    }
}

#[derive(Clone)]
struct ErrorImpl {}

impl ErrorProvider for crate::ErrorImpl {
    fn message_invalid(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        opcode: ClientOpcodeMessage,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?}, invalid message received {opcode}, ") }
    }

    fn username_invalid(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} username invalid") }
    }

    fn username_not_found(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} username not found") }
    }

    fn invalid_password(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid password") }
    }

    fn invalid_integrity_check(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid integrity check") }
    }

    fn invalid_pin(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid pin") }
    }

    fn pin_not_sent(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} pin not sent") }
    }

    fn matrix_card_not_sent(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} matrix card not sent") }
    }

    fn invalid_matrix_card(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid matrix card") }
    }

    fn invalid_user_attempted_reconnect(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid user attempted reconnect") }
    }

    fn invalid_reconnect_integrity_check(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid reconnect integrity check") }
    }

    fn invalid_reconnect_proof(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid reconnect proof") }
    }

    fn invalid_public_key(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        err: InvalidPublicKeyError,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid public key {err}") }
    }

    fn io_error(
        &mut self,
        io: std::io::Error,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {io}") }
    }

    fn provided_file_too_large(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        size: usize,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} provided file too large: {size}") }
    }

    fn transfer_offset_too_large(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        size: u64,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} transfer offset too large: {size}") }
    }

    fn invalid_expected_opcode(
        &mut self,
        err: ExpectedOpcodeError,
        expected_opcode: ExpectedOpcode,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {err} {expected_opcode:?}") }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let should_run = Arc::new(AtomicBool::new(true));

    let should_run_inner = should_run.clone();

    let t = tokio::spawn(async move {
        start_auth_server(
            ProviderImpl {},
            KeyImpl::new(),
            PatchImpl {},
            GameFileImpl {},
            RealmListImpl {},
            ErrorImpl {},
            should_run_inner,
            args.to_options(),
        )
        .await
        .unwrap();
    });

    t.await.unwrap();
}
