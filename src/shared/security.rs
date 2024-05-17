use serde_derive::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct Credentials {
    pub ak: String,
    pub sk: String,
}
