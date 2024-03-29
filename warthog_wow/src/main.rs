use clap::Parser;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warthog_lib::{
    start_auth_server, CMD_AUTH_LOGON_CHALLENGE_Client, CredentialProvider, Credentials,
    GameFileProvider, KeyStorage, MatrixCard, MatrixCardOptions, NormalizedString, Options,
    PatchProvider, Realm, RealmCategory, RealmListProvider, RealmType, SrpServer, SrpVerifier,
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
        }
    }
}

#[derive(Copy, Clone)]
struct ProviderImpl {}

impl CredentialProvider for ProviderImpl {
    fn get_user(&mut self, username: &str) -> impl Future<Output = Option<Credentials>> + Send {
        let v = SrpVerifier::from_username_and_password(
            NormalizedString::new(username).unwrap(),
            NormalizedString::new(username).unwrap(),
        );

        async move {
            const DIGIT_COUNT: u8 = 2;
            const CHALLENGE_COUNT: u8 = 1;
            const HEIGHT: u8 = 8;
            const WIDTH: u8 = 8;

            Some(Credentials {
                password_verifier: *v.password_verifier(),
                salt: *v.salt(),
                pin: Some(1234),
                matrix_card: Some(MatrixCardOptions {
                    matrix_card: MatrixCard::from_data(
                        DIGIT_COUNT,
                        HEIGHT,
                        WIDTH,
                        vec![0; DIGIT_COUNT as usize * HEIGHT as usize * WIDTH as usize],
                    )
                    .unwrap(),
                    challenge_count: CHALLENGE_COUNT,
                }),
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
                    number_of_characters_on_realm: 3,
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    start_auth_server(
        ProviderImpl {},
        KeyImpl::new(),
        PatchImpl {},
        GameFileImpl {},
        RealmListImpl {},
        args.to_options(),
    )
    .await
    .unwrap();
}
