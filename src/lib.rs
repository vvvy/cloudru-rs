pub mod error;
pub mod mauth;
pub mod mauth_obs;
#[macro_use]
pub mod common;
pub mod config;
pub mod fg;
pub mod fg_crt;
pub mod smn;
pub mod obs_s3;
pub mod obs;
pub mod apig;
mod security;

pub use error::{HCError, HCInnerError, Cx};
pub use security::AkSk;
pub use serde_json::Value as JsonValue;
pub use serde_json::to_writer_pretty as json_to_writer_pretty;
pub use config::Config;

pub type Result<T> = std::result::Result<T, HCError>;