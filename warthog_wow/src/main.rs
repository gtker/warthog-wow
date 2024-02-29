use clap::Parser;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warthog_lib::{
    start_auth_server, CMD_AUTH_LOGON_CHALLENGE_Client, CredentialProvider, Credentials,
    GameFileProvider, KeyStorage, NormalizedString, Options, PatchProvider, Realm,
    RealmListProvider, RealmType, SrpServer, SrpVerifier,
};

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
    /// Address to host auth server on.
    #[arg(short, long, default_value = "0.0.0.0:3724")]
    address: SocketAddr,
}

impl Args {
    fn to_options(self) -> Options {
        Options {
            address: self.address,
        }
    }
}

#[derive(Copy, Clone)]
struct ProviderImpl {}

impl CredentialProvider for ProviderImpl {
    fn get_user(&mut self, username: &str) -> Option<Credentials> {
        let v = SrpVerifier::from_username_and_password(
            NormalizedString::new(username).unwrap(),
            NormalizedString::new(username).unwrap(),
        );

        Some(Credentials {
            password_verifier: *v.password_verifier(),
            salt: *v.salt(),
        })
    }

    fn add_user(&mut self, _username: &str, _password: &str) -> Option<()> {
        None
    }
}

#[derive(Clone)]
struct StorageImpl {
    inner: Arc<Mutex<HashMap<String, SrpServer>>>,
}

impl StorageImpl {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Default::default())),
        }
    }
}

impl KeyStorage for StorageImpl {
    fn add_key(&mut self, username: String, server: SrpServer) {
        self.inner.lock().unwrap().insert(username, server);
    }

    fn get_key_for_user(&mut self, username: &str) -> Option<SrpServer> {
        self.inner.lock().unwrap().get(username).cloned()
    }
}

#[derive(Clone)]
struct PatchImpl {}

impl PatchProvider for PatchImpl {
    fn get_patch(&mut self, _message: &CMD_AUTH_LOGON_CHALLENGE_Client) -> Option<Vec<u8>> {
        None
    }
}

#[derive(Clone)]
struct GameFileImpl {}

impl GameFileProvider for GameFileImpl {
    fn get_game_files(&mut self, _message: &CMD_AUTH_LOGON_CHALLENGE_Client) -> Option<Vec<u8>> {
        None
    }
}

#[derive(Clone)]
struct RealmListImpl {}

impl RealmListProvider for RealmListImpl {
    fn get_realm_list(&mut self, _message: &CMD_AUTH_LOGON_CHALLENGE_Client) -> Vec<Realm> {
        vec![Realm {
            realm_type: RealmType::PlayerVsEnvironment,
            locked: false,
            flag: Default::default(),
            name: "Test Realm".to_string(),
            address: "localhost:8085".to_string(),
            population: Default::default(),
            number_of_characters_on_realm: 3,
            category: Default::default(),
            realm_id: 0,
        }]
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    start_auth_server(
        ProviderImpl {},
        StorageImpl::new(),
        PatchImpl {},
        GameFileImpl {},
        RealmListImpl {},
        args.to_options(),
    )
    .await;
}
