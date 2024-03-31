use crate::auth::error::InternalError;
use crate::ExpectedOpcode;
use std::sync::Arc;
use tokio::net::TcpStream;
use wow_login_messages::all::CMD_AUTH_LOGON_CHALLENGE_Client;
use wow_login_messages::version_8::opcodes::ClientOpcodeMessage;
use wow_login_messages::version_8::CMD_AUTH_LOGON_CHALLENGE_Server;
use wow_login_messages::version_8::{CMD_XFER_DATA, CMD_XFER_INITIATE};
use wow_login_messages::{CollectiveMessage, Message};

pub(crate) async fn transfer(
    mut stream: TcpStream,
    c: CMD_AUTH_LOGON_CHALLENGE_Client,
    data: Arc<[u8]>,
    file_size: u64,
) -> Result<(), InternalError> {
    CMD_AUTH_LOGON_CHALLENGE_Server::LoginDownloadFile
        .tokio_write_protocol(&mut stream, c.protocol_version)
        .await?;

    let file_md5 = md5::compute(&data).0;

    CMD_XFER_INITIATE {
        filename: "Patch".to_string(),
        file_size,
        file_md5,
    }
    .tokio_write(&mut stream)
    .await?;

    let s = match ClientOpcodeMessage::tokio_read_protocol(&mut stream, c.protocol_version).await {
        Ok(s) => s,
        Err(err) => {
            return Err(InternalError::ExpectedOpcodeError {
                err,
                expected: ExpectedOpcode::XferOrResume,
            });
        }
    };

    let offset = match s {
        ClientOpcodeMessage::CMD_XFER_ACCEPT => 0,
        ClientOpcodeMessage::CMD_XFER_RESUME(r) => match r.offset.try_into() {
            Ok(e) => e,
            Err(_) => {
                return Err(InternalError::TransferOffsetTooLarge {
                    message: c,
                    size: r.offset,
                })
            }
        },
        opcode => return Err(InternalError::MessageInvalid { message: c, opcode }),
    };

    const TRANSFER_CHUNK: usize = 64;

    for i in (offset..data.len()).step_by(TRANSFER_CHUNK) {
        let mut buf = [0_u8; 1];
        let size = stream.peek(&mut buf).await?;
        if size != 0 {
            // Client doesn't send any messages other than CMD_XFER_CANCEL
            return Ok(());
        }

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

    // Keep the connection alive until the client breaks it off and updates
    while let Ok(m) = ClientOpcodeMessage::tokio_read(&mut stream).await {
        dbg!(m);
    }

    Ok(())
}
