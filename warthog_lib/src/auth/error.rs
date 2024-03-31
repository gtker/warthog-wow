use crate::ExpectedOpcode;
use wow_login_messages::all::{
    CMD_AUTH_LOGON_CHALLENGE_Client, CMD_AUTH_RECONNECT_CHALLENGE_Client,
};
use wow_login_messages::errors::ExpectedOpcodeError;
use wow_login_messages::version_8::opcodes::ClientOpcodeMessage;
use wow_srp::error::InvalidPublicKeyError;

#[derive(Debug)]
pub(crate) enum InternalError {
    MessageInvalid {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        opcode: ClientOpcodeMessage,
    },
    UsernameInvalid {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
    },
    UsernameNotFound {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
    },
    InvalidPasswordForUser {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
    },
    InvalidIntegrityCheckForUser {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
    },
    PinInvalidForUser {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
    },
    PinNotSentForUser {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
    },
    MatrixCardDataNotSentForUser {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
    },
    MatrixCardInvalidForUser {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
    },
    InvalidUserAttemptedReconnect {
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
    },
    InvalidReconnectIntegrityCheckForUser {
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
    },
    InvalidReconnectProofForUser {
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
    },
    InvalidPublicKey {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        err: InvalidPublicKeyError,
    },
    ExpectedOpcodeError {
        expected: ExpectedOpcode,
        err: ExpectedOpcodeError,
    },
    ProvidedFileTooLarge {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        size: usize,
    },
    TransferOffsetTooLarge {
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        size: u64,
    },
    Io {
        err: std::io::Error,
    },
}

impl From<std::io::Error> for InternalError {
    fn from(value: std::io::Error) -> Self {
        Self::Io { err: value }
    }
}
