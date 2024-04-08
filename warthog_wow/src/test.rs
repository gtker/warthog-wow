use crate::{lib_main, ApplicationOptions};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use warthog_lib::{CMD_AUTH_LOGON_CHALLENGE_Client, Options, Population};
use warthog_messages::ClientOpcodes;
use wow_client::{connect_and_authenticate, Locale, Os, Platform, ProtocolVersion, Version};

fn vanilla_1_12(account_name: String) -> CMD_AUTH_LOGON_CHALLENGE_Client {
    CMD_AUTH_LOGON_CHALLENGE_Client {
        protocol_version: ProtocolVersion::Three,
        version: Version {
            major: 1,
            minor: 12,
            patch: 1,
            build: 5875,
        },
        platform: Platform::X86,
        os: Os::Windows,
        locale: Locale::EnGb,
        utc_timezone_offset: 60,
        client_ip_address: Ipv4Addr::new(127, 0, 0, 1),
        account_name,
    }
}

async fn register_realm(mut stream: &mut TcpStream, name: String, address: String) -> u8 {
    warthog_messages::ServerOpcodes::RegisterRealm {
        name,
        address,
        population: 200.0,
        locked: false,
        flags: 0,
        category: 0,
        realm_type: 0,
        version_major: 0,
        version_minor: 0,
        version_patch: 0,
        version_build: 0,
    }
    .tokio_write(&mut stream)
    .await
    .unwrap();

    match ClientOpcodes::tokio_read(&mut stream).await.unwrap() {
        ClientOpcodes::SessionKeyAnswer { .. } => panic!(),
        ClientOpcodes::RegisterRealmReply { realm_id } => realm_id.unwrap(),
    }
}

async fn start_server(
    options: Options,
    application_options: ApplicationOptions,
) -> (Arc<AtomicBool>, JoinHandle<()>) {
    let game_address = options.address;

    let should_run = Arc::new(AtomicBool::new(true));
    let should_run_inner = should_run.clone();
    let main =
        tokio::spawn(async move { lib_main(options, application_options, should_run_inner).await });

    let mut i = 0;
    while TcpStream::connect(game_address).await.is_err() {
        assert_ne!(i, 20);

        tokio::time::sleep(Duration::new(0, 10)).await;
        i += 1;
    }

    (should_run, main)
}

#[tokio::test]
async fn register_realms() {
    const REPLY_PORT: u16 = 32657;
    const GAME_PORT: u16 = REPLY_PORT + 1;

    const REPLY_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), REPLY_PORT);
    const GAME_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), GAME_PORT);

    const APPLICATION_OPTIONS: ApplicationOptions = ApplicationOptions {
        reply_address: REPLY_ADDRESS,
        use_pin: false,
        use_matrix_card: false,
    };

    const OPTIONS: Options = Options {
        address: GAME_ADDRESS,
        randomize_pin_grid: false,
        max_concurrent_users: 10000,
    };

    let (should_run, main) = start_server(OPTIONS, APPLICATION_OPTIONS).await;

    {
        let (_, realms, _) =
            connect_and_authenticate(vanilla_1_12("A".to_string()), GAME_ADDRESS, "A")
                .await
                .unwrap();

        assert!(realms.is_empty());
    }

    const REALM_NAME: &str = "Test Realm";
    const REALM_ADDRESS: &str = "localhost:8085";

    let mut reply = TcpStream::connect(REPLY_ADDRESS).await.unwrap();
    let realm_id = register_realm(
        &mut reply,
        REALM_NAME.to_string(),
        REALM_ADDRESS.to_string(),
    )
    .await;

    {
        let (_, realms, _) =
            connect_and_authenticate(vanilla_1_12("A".to_string()), GAME_ADDRESS, "A")
                .await
                .unwrap();

        match realms.as_slice() {
            [realm] => {
                assert_eq!(realm.population, Population::from(200.0));
                assert_eq!(realm.locked, false);
                assert_eq!(realm.name, REALM_NAME);
                assert_eq!(realm.address, REALM_ADDRESS);
                assert_eq!(realm.realm_id, realm_id);
            }
            _ => panic!(),
        }
        assert!(!realms.is_empty());
    }

    const REALM2_NAME: &str = "Test Realm2";
    const REALM2_ADDRESS: &str = "localhost:8088";

    let mut reply2 = TcpStream::connect(REPLY_ADDRESS).await.unwrap();
    let realm_id2 = register_realm(
        &mut reply2,
        REALM2_NAME.to_string(),
        REALM2_ADDRESS.to_string(),
    )
    .await;

    {
        let (_, realms, _) =
            connect_and_authenticate(vanilla_1_12("A".to_string()), GAME_ADDRESS, "A")
                .await
                .unwrap();

        match realms.as_slice() {
            [realm, realm2] => {
                let (first, second) = if realm.realm_id == 0 {
                    (&realm, &realm2)
                } else {
                    (&realm2, &realm)
                };

                assert_eq!(first.population, Population::from(200.0));
                assert_eq!(first.locked, false);
                assert_eq!(first.name, REALM_NAME);
                assert_eq!(first.address, REALM_ADDRESS);
                assert_eq!(first.realm_id, realm_id);

                assert_eq!(second.population, Population::from(200.0));
                assert_eq!(second.locked, false);
                assert_eq!(second.name, REALM2_NAME);
                assert_eq!(second.address, REALM2_ADDRESS);
                assert_eq!(second.realm_id, realm_id2);
            }
            _ => panic!(),
        }
        assert!(!realms.is_empty());
    }

    should_run.store(false, Ordering::SeqCst);
    main.await.unwrap();
}
