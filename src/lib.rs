mod model;
mod shared;

pub mod error;

pub mod nonblocking;
pub mod blocking;

pub use error::{CloudRuError, Cx};
pub use shared::{config::{self, Config}, security::Credentials};
pub use serde_json::Value as JsonValue;
pub use serde_json::to_writer_pretty as json_to_writer_pretty;

pub type Result<T> = std::result::Result<T, CloudRuError>;