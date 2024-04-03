mod credentials;
mod errors;
mod game_files;
mod keys;
mod patches;
mod realm_list;
mod reply;

use crate::reply::start_reply_server;
use clap::Parser;
use credentials::ProviderImpl;
use errors::ErrorImpl;
use game_files::GameFileImpl;
use keys::KeyImpl;
use patches::PatchImpl;
use realm_list::RealmListImpl;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use warthog_lib::{start_auth_server, Options};

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
    /// Address to host auth server on.
    #[arg(short, long, default_value = "0.0.0.0:3724")]
    address: SocketAddr,
    /// Randomize PIN grid number locations.
    #[arg(short, long, default_value = "false")]
    pin_grid_randomize: bool,
    /// Address to reply to inter server communication on.
    #[arg(short, long, default_value = "0.0.0.0:8086")]
    reply_address: SocketAddr,
}

impl Args {
    fn to_options(self) -> (Options, SocketAddr) {
        (
            Options {
                address: self.address,
                randomize_pin_grid: self.pin_grid_randomize,
                max_concurrent_users: 1000,
            },
            self.reply_address,
        )
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let should_run = Arc::new(AtomicBool::new(true));

    let should_run_inner = should_run.clone();

    let keys = KeyImpl::new();
    let realms = RealmListImpl::new();

    let (options, reply_address) = args.to_options();

    let keys_auth = keys.clone();
    let realms_auth = realms.clone();
    let auth = tokio::spawn(async move {
        start_auth_server(
            ProviderImpl {},
            keys_auth,
            PatchImpl {},
            GameFileImpl {},
            realms_auth,
            ErrorImpl {},
            should_run_inner,
            options,
        )
        .await
    });

    let reply = tokio::spawn(async move { start_reply_server(keys, realms, reply_address).await });

    tokio::select! {
        auth = auth => {
            println!("auth terminated {auth:?}");
        }
        reply = reply => {
            println!("reply terminated {reply:?}");
        }
    }
}
