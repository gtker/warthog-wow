use crate::auth::error::InternalError;
use crate::auth::send_realm_list;
use crate::{ExpectedOpcode, KeyStorage, RealmListProvider};
use tokio::net::TcpStream;
use wow_login_messages::all::{
    CMD_AUTH_LOGON_CHALLENGE_Client, CMD_AUTH_RECONNECT_CHALLENGE_Client,
};
use wow_login_messages::helper::tokio_expect_client_message_protocol;
use wow_login_messages::version_8::{
    CMD_AUTH_RECONNECT_CHALLENGE_Server, CMD_AUTH_RECONNECT_PROOF_Client,
    CMD_AUTH_RECONNECT_PROOF_Server, LoginResult,
};
use wow_login_messages::CollectiveMessage;

pub(crate) async fn reconnect(
    mut storage: impl KeyStorage,
    realm_list_provider: impl RealmListProvider,
    mut stream: TcpStream,
    c: CMD_AUTH_RECONNECT_CHALLENGE_Client,
) -> Result<(), InternalError> {
    let Some(mut server) = storage.get_key_for_user(&c.account_name).await else {
        CMD_AUTH_RECONNECT_CHALLENGE_Server::FailUnknownAccount
            .tokio_write_protocol(&mut stream, c.protocol_version)
            .await?;

        return Err(InternalError::InvalidUserAttemptedReconnect { message: c });
    };

    CMD_AUTH_RECONNECT_CHALLENGE_Server::Success {
        challenge_data: *server.reconnect_challenge_data(),
        // Unused on 1.12
        checksum_salt: [0; 16],
    }
    .tokio_write_protocol(&mut stream, c.protocol_version)
    .await?;

    let s = match tokio_expect_client_message_protocol::<CMD_AUTH_RECONNECT_PROOF_Client, _>(
        &mut stream,
        c.protocol_version,
    )
    .await
    {
        Ok(s) => s,
        Err(err) => {
            return Err(InternalError::ExpectedOpcodeError {
                err,
                expected: ExpectedOpcode::ReconnectProof,
            })
        }
    };

    if s.client_checksum != wow_srp::integrity::reconnect_integrity_check(&s.proof_data) {
        CMD_AUTH_RECONNECT_PROOF_Server {
            result: LoginResult::FailVersionInvalid,
        }
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

        return Err(InternalError::InvalidReconnectIntegrityCheckForUser { message: c });
    }

    if !server.verify_reconnection_attempt(s.proof_data, s.client_proof) {
        CMD_AUTH_RECONNECT_PROOF_Server {
            result: LoginResult::FailIncorrectPassword,
        }
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

        return Err(InternalError::InvalidReconnectProofForUser { message: c });
    }

    CMD_AUTH_RECONNECT_PROOF_Server {
        result: LoginResult::Success,
    }
    .tokio_write_protocol(&mut stream, c.protocol_version)
    .await?;

    let c = CMD_AUTH_LOGON_CHALLENGE_Client {
        protocol_version: c.protocol_version,
        version: c.version,
        platform: c.platform,
        os: c.os,
        locale: c.locale,
        utc_timezone_offset: c.utc_timezone_offset,
        client_ip_address: c.client_ip_address,
        account_name: c.account_name,
    };

    send_realm_list(&mut stream, &c, realm_list_provider).await?;

    Ok(())
}
