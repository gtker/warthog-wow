use std::future::Future;
use warthog_lib::{
    CMD_AUTH_LOGON_CHALLENGE_Client, Realm, RealmCategory, RealmListProvider, RealmType,
};

#[derive(Clone)]
pub(crate) struct RealmListImpl {
    realms: Vec<Realm>,
}

impl RealmListImpl {
    pub fn new() -> Self {
        Self {
            realms: vec![
                Realm {
                    realm_type: RealmType::PlayerVsEnvironment,
                    locked: false,
                    flag: Default::default(),
                    name: "Test Realm2".to_string(),
                    address: "localhost:8085".to_string(),
                    population: Default::default(),
                    number_of_characters_on_realm: 2,
                    category: RealmCategory::One,
                    realm_id: 1,
                },
                Realm {
                    realm_type: RealmType::PlayerVsEnvironment,
                    locked: false,
                    flag: Default::default(),
                    name: "Test Realm".to_string(),
                    address: "localhost:8085".to_string(),
                    population: Default::default(),
                    number_of_characters_on_realm: 3,
                    category: RealmCategory::Two,
                    realm_id: 0,
                },
            ],
        }
    }
}

impl RealmListProvider for RealmListImpl {
    fn get_realm_list(
        &mut self,
        _message: &CMD_AUTH_LOGON_CHALLENGE_Client,
    ) -> impl Future<Output = Vec<Realm>> + Send {
        async move { self.realms.clone() }
    }
}
