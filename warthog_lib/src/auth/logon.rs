use crate::auth::send_realm_list;
use crate::{CredentialProvider, GameFileProvider, KeyStorage, RealmListProvider};
use anyhow::anyhow;
use tokio::net::TcpStream;
use wow_login_messages::all::CMD_AUTH_LOGON_CHALLENGE_Client;
use wow_login_messages::helper::tokio_expect_client_message_protocol;
use wow_login_messages::version_8::{
    AccountFlag, CMD_AUTH_LOGON_CHALLENGE_Server, CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult,
    CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag, CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag_Pin,
    CMD_AUTH_LOGON_PROOF_Client, CMD_AUTH_LOGON_PROOF_Server,
    CMD_AUTH_LOGON_PROOF_Server_LoginResult,
};
use wow_login_messages::CollectiveMessage;
use wow_srp::normalized_string::NormalizedString;
use wow_srp::pin::{get_pin_grid_seed, get_pin_salt};
use wow_srp::server::SrpVerifier;
use wow_srp::{PublicKey, GENERATOR, LARGE_SAFE_PRIME_LITTLE_ENDIAN};

pub(crate) async fn logon(
    mut provider: impl CredentialProvider,
    mut storage: impl KeyStorage,
    mut game_file_provider: impl GameFileProvider,
    realm_list_provider: impl RealmListProvider,
    mut stream: TcpStream,
    c: CMD_AUTH_LOGON_CHALLENGE_Client,
) -> anyhow::Result<()> {
    let Ok(username) = NormalizedString::new(&c.account_name) else {
        CMD_AUTH_LOGON_CHALLENGE_Server {
            result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::FailUnknownAccount,
        }
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

        return Err(anyhow!("username '{}' is invalid", &c.account_name));
    };

    let Some(credentials) = provider.get_user(&c.account_name).await else {
        CMD_AUTH_LOGON_CHALLENGE_Server {
            result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::FailUnknownAccount,
        }
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

        return Err(anyhow!("username '{}' not found", &c.account_name));
    };

    let verifier = SrpVerifier::from_database_values(
        username,
        credentials.password_verifier,
        credentials.salt,
    );
    let proof = verifier.into_proof();

    let crc_salt = wow_srp::integrity::get_salt_value();

    let pin_grid_seed = get_pin_grid_seed();
    let pin_salt = get_pin_salt();

    CMD_AUTH_LOGON_CHALLENGE_Server {
        result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::Success {
            crc_salt,
            generator: vec![GENERATOR],
            large_safe_prime: LARGE_SAFE_PRIME_LITTLE_ENDIAN.to_vec(),
            salt: *proof.salt(),
            security_flag: CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag::new_pin(
                CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag_Pin {
                    pin_grid_seed,
                    pin_salt,
                },
            ),
            server_public_key: *proof.server_public_key(),
        },
    }
    .tokio_write_protocol(&mut stream, c.protocol_version)
    .await?;

    let p = {
        let mut p = dbg!(get_pin_grid_seed());
        while p < 1000 {
            p = dbg!(get_pin_grid_seed());
        }
        p
    };

    let s = tokio_expect_client_message_protocol::<CMD_AUTH_LOGON_PROOF_Client, _>(
        &mut stream,
        c.protocol_version,
    )
    .await?;
    dbg!(&s);

    if let Some(pin) = s.security_flag.get_pin() {
        if let Some(hash) =
            wow_srp::pin::calculate_hash(dbg!(p), pin_grid_seed, &pin_salt, &pin.pin_salt)
        {
            if hash != pin.pin_hash {
                println!("PIN hashes do not match");
            } else {
                println!("PIN hashes match");
            }
        }
    }

    if let Some(game_files) = game_file_provider.get_game_files(&c).await {
        if wow_srp::integrity::login_integrity_check_generic(
            &game_files,
            &crc_salt,
            &s.client_public_key,
        ) != s.crc_hash
        {
            CMD_AUTH_LOGON_PROOF_Server {
                result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::FailVersionInvalid,
            }
            .tokio_write_protocol(&mut stream, c.protocol_version)
            .await?;

            return Err(anyhow!("invalid integrity check for '{}'", c.account_name));
        }
    }

    let client_public_key = PublicKey::from_le_bytes(s.client_public_key)?;
    let Ok((server, proof)) = proof.into_server(client_public_key, s.client_proof) else {
        CMD_AUTH_LOGON_PROOF_Server {
            result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::FailIncorrectPassword,
        }
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

        return Err(anyhow!("invalid password for {}", c.account_name));
    };

    storage.add_key(c.account_name.clone(), server).await;

    CMD_AUTH_LOGON_PROOF_Server {
        result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::Success {
            account_flag: AccountFlag::empty(),
            hardware_survey_id: 0,
            server_proof: proof,
            unknown: 0,
        },
    }
    .tokio_write_protocol(&mut stream, c.protocol_version)
    .await?;

    send_realm_list(&mut stream, &c, realm_list_provider).await?;

    Ok(())
}
