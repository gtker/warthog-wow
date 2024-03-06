use crate::{CredentialProvider, KeyStorage};
use anyhow::anyhow;
use std::sync::Arc;
use tokio::net::TcpStream;
use wow_login_messages::all::CMD_AUTH_LOGON_CHALLENGE_Client;
use wow_login_messages::version_8::opcodes::ClientOpcodeMessage;
use wow_login_messages::version_8::{
    CMD_AUTH_LOGON_CHALLENGE_Server, CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult,
};
use wow_login_messages::version_8::{CMD_XFER_DATA, CMD_XFER_INITIATE};
use wow_login_messages::{CollectiveMessage, Message};

pub(crate) async fn transfer(
    _provider: impl CredentialProvider,
    _storage: impl KeyStorage,
    mut stream: TcpStream,
    c: CMD_AUTH_LOGON_CHALLENGE_Client,
    data: Arc<[u8]>,
) -> anyhow::Result<()> {
    CMD_AUTH_LOGON_CHALLENGE_Server {
        result: CMD_AUTH_LOGON_CHALLENGE_Server_LoginResult::LoginDownloadFile,
    }
    .tokio_write_protocol(&mut stream, c.protocol_version)
    .await?;

    let file_md5 = md5::compute(&data).0;

    CMD_XFER_INITIATE {
        filename: "Patch".to_string(),
        file_size: data.len().try_into()?,
        file_md5,
    }
    .tokio_write(&mut stream)
    .await?;

    let s = ClientOpcodeMessage::tokio_read_protocol(&mut stream, c.protocol_version).await?;

    let offset = match s {
        ClientOpcodeMessage::CMD_XFER_ACCEPT => 0,
        ClientOpcodeMessage::CMD_XFER_RESUME(c) => c.offset.try_into()?,
        s => return Err(anyhow!("invalid opcode: {s}")),
    };

    const TRANSFER_CHUNK: usize = 64;

    for i in (offset..data.len()).step_by(TRANSFER_CHUNK) {
        let length = if i + TRANSFER_CHUNK > data.len() {
            data.len() - i
        } else {
            TRANSFER_CHUNK
        };

        CMD_XFER_DATA {
            data: data[i..i + length].to_vec(),
        }
        .tokio_write(&mut stream)
        .await?;
    }

    while let Ok(m) = ClientOpcodeMessage::tokio_read(&mut stream).await {
        dbg!(m);
    }

    Ok(())
}
