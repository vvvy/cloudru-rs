use std::sync::Arc;

use crate::shared::config::*;
use super::*;
use crate::*;

pub type Client = crate::shared::client::Client<Arc<HttpClient>>;

pub trait ServiceClientBuild {
    fn obs(&self) -> Result<super::obs::ObsClient>;
    fn apig(&self) -> Result<super::apig::ApigClient>;
    fn fg(&self) -> Result<super::fg::FgClient>;
}

impl ServiceClientBuild for Client {
    fn obs(&self) -> Result<super::obs::ObsClient> { Ok(super::obs::ObsClient::new(
        self.resolve_endpoint(svc_id::obs)?,
        self.credentials.clone(),
        self.http_client.clone()))
    }
    fn apig(&self) -> Result<super::apig::ApigClient> { Ok(super::apig::ApigClient::new(
        self.resolve_endpoint(svc_id::apig)?, 
        self.credentials.clone(),
        self.http_client.clone()))
    }
    fn fg(&self) -> Result<super::fg::FgClient> { Ok(super::fg::FgClient::new(
        self.resolve_endpoint(svc_id::fg)?,
        self.resolve_project_id()?,
        self.credentials.clone(),
        self.http_client.clone()))
    }
}

pub use crate::shared::client::ClientBuilder;

pub trait ClientBuild {
    fn build(self) -> Result<Client>;
}

impl ClientBuild for ClientBuilder {
    fn build(self) -> Result<Client> {
        let http_client = Arc::new(HttpClient::new());
        self.build_with_http_client(http_client)
    }
}
