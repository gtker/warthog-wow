use std::future::Future;
use warthog_lib::{
    CMD_AUTH_LOGON_CHALLENGE_Client, CredentialProvider, Credentials, MatrixCard,
    MatrixCardOptions, NormalizedString, SrpVerifier,
};

#[derive(Copy, Clone)]
pub(crate) struct ProviderImpl {}

const DIGIT_COUNT: u8 = 2;
const CHALLENGE_COUNT: u8 = 1;
const HEIGHT: u8 = 8;
const WIDTH: u8 = 8;

impl CredentialProvider for ProviderImpl {
    fn get_user(
        &mut self,
        username: &str,
        message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Credentials>> + Send {
        let v = SrpVerifier::from_username_and_password(
            NormalizedString::new(username).unwrap(),
            NormalizedString::new(username).unwrap(),
        );

        let matrix_card = if message.version.supports_matrix_card() {
            Some(MatrixCardOptions {
                matrix_card: MatrixCard::from_data(
                    DIGIT_COUNT,
                    HEIGHT,
                    WIDTH,
                    vec![0; DIGIT_COUNT as usize * HEIGHT as usize * WIDTH as usize],
                )
                .unwrap(),
                challenge_count: CHALLENGE_COUNT,
            })
        } else {
            None
        };

        let pin = if message.version.supports_pin() {
            Some(1234)
        } else {
            None
        };

        async move {
            Some(Credentials {
                password_verifier: *v.password_verifier(),
                salt: *v.salt(),
                pin,
                matrix_card,
            })
        }
    }

    fn add_user(
        &mut self,
        _username: &str,
        _password: &str,
    ) -> impl Future<Output = Option<()>> + Send {
        async move { None }
    }
}
