use crate::auth::send_realm_list;
use crate::{
    CredentialProvider, Credentials, GameFileProvider, KeyStorage, Options, RealmListProvider,
};
use anyhow::anyhow;
use tokio::net::TcpStream;
use wow_login_messages::all::CMD_AUTH_LOGON_CHALLENGE_Client;
use wow_login_messages::helper::tokio_expect_client_message_protocol;
use wow_login_messages::version_8::{
    AccountFlag, CMD_AUTH_LOGON_CHALLENGE_Server, CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult,
    CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag,
    CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag_MatrixCard,
    CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag_Pin, CMD_AUTH_LOGON_PROOF_Client,
    CMD_AUTH_LOGON_PROOF_Server, CMD_AUTH_LOGON_PROOF_Server_LoginResult,
};
use wow_login_messages::CollectiveMessage;
use wow_srp::matrix_card::{get_matrix_card_seed, verify_matrix_card_hash};
use wow_srp::normalized_string::NormalizedString;
use wow_srp::pin::{get_pin_grid_seed, get_pin_salt};
use wow_srp::server::{SrpServer, SrpVerifier};
use wow_srp::{PublicKey, GENERATOR, LARGE_SAFE_PRIME_LITTLE_ENDIAN};

pub(crate) async fn logon(
    mut provider: impl CredentialProvider,
    mut storage: impl KeyStorage,
    mut game_file_provider: impl GameFileProvider,
    realm_list_provider: impl RealmListProvider,
    mut stream: TcpStream,
    c: CMD_AUTH_LOGON_CHALLENGE_Client,
    options: &Options,
) -> anyhow::Result<()> {
    let Ok(username) = NormalizedString::new(&c.account_name) else {
        CMD_AUTH_LOGON_CHALLENGE_Server {
            result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::FailUnknownAccount,
        }
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

        return Err(anyhow!("username '{}' is invalid", &c.account_name));
    };

    let Some(credentials) = provider.get_user(&c.account_name, &c).await else {
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

    let mut security_flag = CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag::empty();
    let pin_grid_seed = if options.randomize_pin_grid {
        get_pin_grid_seed()
    } else {
        0
    };
    let pin_salt = get_pin_salt();

    if credentials.pin.is_some() {
        security_flag = security_flag.set_pin(CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag_Pin {
            pin_grid_seed,
            pin_salt,
        });
    }

    let seed = get_matrix_card_seed();

    if let Some(c) = &credentials.matrix_card {
        security_flag = security_flag.set_matrix_card(
            CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag_MatrixCard {
                challenge_count: c.challenge_count,
                digit_count: c.matrix_card.digit_count(),
                height: c.matrix_card.height(),
                seed,
                width: c.matrix_card.width(),
            },
        );
    }

    CMD_AUTH_LOGON_CHALLENGE_Server {
        result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::Success {
            crc_salt,
            generator: vec![GENERATOR],
            large_safe_prime: LARGE_SAFE_PRIME_LITTLE_ENDIAN.to_vec(),
            salt: *proof.salt(),
            security_flag,
            server_public_key: *proof.server_public_key(),
        },
    }
    .tokio_write_protocol(&mut stream, c.protocol_version)
    .await?;

    let s = tokio_expect_client_message_protocol::<CMD_AUTH_LOGON_PROOF_Client, _>(
        &mut stream,
        c.protocol_version,
    )
    .await?;

    let client_public_key = PublicKey::from_le_bytes(s.client_public_key)?;
    let Ok((server, server_proof)) = proof.into_server(client_public_key, s.client_proof) else {
        CMD_AUTH_LOGON_PROOF_Server {
            result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::FailIncorrectPassword,
        }
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

        return Err(anyhow!("invalid password for {}", c.account_name));
    };

    if let Err(err) = check_2fa_login_details(
        &c.account_name,
        credentials,
        pin_grid_seed,
        &pin_salt,
        seed,
        &s,
        &server,
    )
    .await
    {
        CMD_AUTH_LOGON_PROOF_Server {
            result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::FailIncorrectPassword,
        }
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

        return Err(err);
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

    storage.add_key(c.account_name.clone(), server).await;

    CMD_AUTH_LOGON_PROOF_Server {
        result: CMD_AUTH_LOGON_PROOF_Server_LoginResult::Success {
            account_flag: AccountFlag::empty(),
            hardware_survey_id: 0,
            server_proof,
            unknown: 0,
        },
    }
    .tokio_write_protocol(&mut stream, c.protocol_version)
    .await?;

    send_realm_list(&mut stream, &c, realm_list_provider).await?;

    Ok(())
}

async fn check_2fa_login_details(
    account_name: &str,
    credentials: Credentials,
    pin_grid_seed: u32,
    pin_salt: &[u8; 16],
    seed: u64,
    s: &CMD_AUTH_LOGON_PROOF_Client,
    server: &SrpServer,
) -> anyhow::Result<()> {
    if let Some(p) = credentials.pin {
        if let Some(pin) = s.security_flag.get_pin() {
            if wow_srp::pin::verify_client_pin_hash(
                p,
                pin_grid_seed,
                &pin_salt,
                &pin.pin_salt,
                &pin.pin_hash,
            ) {
                println!("PIN hashes match");
            } else {
            }
        } else {
            return Err(anyhow!("{account_name} did not send PIN when required"));
        }
    }

    if let Some(cred_card) = credentials.matrix_card {
        if let Some(card) = s.security_flag.get_matrix_card() {
            let client_proof = card.matrix_card_proof;

            if !verify_matrix_card_hash(
                &cred_card.matrix_card,
                cred_card.challenge_count,
                seed,
                server.session_key(),
                &client_proof,
            ) {
                return Err(anyhow!(
                    "{account_name} does not have matrix card data in the correct format",
                ));
            } else {
                println!("{account_name} passed matrix card.");
            }
        } else {
            return Err(anyhow!(
                "{account_name} did not send matrix card when required",
            ));
        }
    }

    Ok(())
}
