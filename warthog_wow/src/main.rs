mod credentials;
mod errors;
mod game_files;
mod keys;
mod patches;
mod realm_list;

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
