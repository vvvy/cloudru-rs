use serde_derive::Deserialize;

pub use s3::bucket::Bucket;
pub use s3::creds::Credentials;
pub use s3::Region;

#[derive(Clone, Deserialize)]
pub struct ObsConfig {
    pub endpoint: String,
    pub region: String,
}

pub fn credentials(aksk: &crate::AkSk) -> crate::Result<Credentials> {
    Ok(Credentials::new(Some(&aksk.ak), Some(&aksk.sk), None, None, None)?)
}
