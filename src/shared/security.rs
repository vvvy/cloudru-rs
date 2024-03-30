use serde_derive::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct AkSk {
    pub ak: String,
    pub sk: String,
}
