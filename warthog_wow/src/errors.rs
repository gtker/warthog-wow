use std::future::Future;
use std::net::SocketAddr;
use warthog_lib::{
    CMD_AUTH_LOGON_CHALLENGE_Client, CMD_AUTH_RECONNECT_CHALLENGE_Client, ClientOpcodeMessage,
    ErrorProvider, ExpectedOpcode, ExpectedOpcodeError, InvalidPublicKeyError,
};

#[derive(Clone, Debug)]
pub(crate) struct ErrorImpl {}

impl ErrorProvider for ErrorImpl {
    fn message_invalid(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        opcode: ClientOpcodeMessage,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?}, invalid message received {opcode}, ") }
    }

    fn username_invalid(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} username invalid") }
    }

    fn username_not_found(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} username not found") }
    }

    fn invalid_password(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid password") }
    }

    fn invalid_integrity_check(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid integrity check") }
    }

    fn invalid_pin(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid pin") }
    }

    fn pin_not_sent(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} pin not sent") }
    }

    fn matrix_card_not_sent(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} matrix card not sent") }
    }

    fn invalid_matrix_card(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid matrix card") }
    }

    fn invalid_user_attempted_reconnect(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid user attempted reconnect") }
    }

    fn invalid_reconnect_integrity_check(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid reconnect integrity check") }
    }

    fn invalid_reconnect_proof(
        &mut self,
        message: CMD_AUTH_RECONNECT_CHALLENGE_Client,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid reconnect proof") }
    }

    fn invalid_public_key(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        err: InvalidPublicKeyError,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} invalid public key {err}") }
    }

    fn io_error(
        &mut self,
        io: std::io::Error,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {io}") }
    }

    fn provided_file_too_large(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        size: usize,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} provided file too large: {size}") }
    }

    fn transfer_offset_too_large(
        &mut self,
        message: CMD_AUTH_LOGON_CHALLENGE_Client,
        size: u64,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {message:?} transfer offset too large: {size}") }
    }

    fn invalid_expected_opcode(
        &mut self,
        err: ExpectedOpcodeError,
        expected_opcode: ExpectedOpcode,
        addr: SocketAddr,
    ) -> impl Future<Output = ()> + Send {
        async move { println!("{addr}, {err} {expected_opcode:?}") }
    }
}
