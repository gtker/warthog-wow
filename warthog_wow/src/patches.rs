use std::future::Future;
use std::sync::Arc;
use warthog_lib::{CMD_AUTH_LOGON_CHALLENGE_Client, PatchProvider};

#[derive(Clone)]
pub(crate) struct PatchImpl {}

impl PatchProvider for PatchImpl {
    fn get_patch(
        &mut self,
        _message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Option<Arc<[u8]>>> + Send {
        async move { None }
    }
}
