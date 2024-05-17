//!Function Graph-related api
use std::sync::Arc;

use super::*;
use crate::*;

pub use crate::model::fg as model;

pub struct FgClient {
    endpoint: String,
    project_id: String,
    credentials: Credentials,
    http_client: Arc<HttpClient>,
}

impl FgClient {
    pub fn new(endpoint: String, project_id: String, credentials: Credentials, http_client: Arc<HttpClient>) -> Self { Self { endpoint, project_id, credentials, http_client } }
    pub fn logging_to_lts_enable(&self) -> Result<JsonValue> { 
        logging_to_lts_enable(&self.endpoint, &self.project_id, &self.credentials, &self.http_client) 
    }
    pub fn logging_to_lts_detail(&self, urn: &str) -> Result<JsonValue> {
        logging_to_lts_detail(&self.endpoint, &self.project_id, urn, &self.credentials, &self.http_client)
    }
}


pub fn logging_to_lts_enable(
    fg_endpoint: &str,
    project_id: &str,
    credentials: &Credentials,
    client: &HttpClient
) -> Result<JsonValue> {
    //POST /v2/{project_id}/fgs/functions/enable-lts-logs
    api_call!(POST /"{fg_endpoint}/v2/{project_id}/fgs/functions/enable-lts-logs" ;
        credentials,
        client
    )
}

pub fn logging_to_lts_detail(
    fg_endpoint: &str,
    project_id: &str,
    urn: &str,
    credentials: &Credentials,
    client: &HttpClient
) -> Result<JsonValue> {
    //GET /v2/{project_id}/fgs/functions/{urn}/lts-log-detail
    api_call!(GET /"{fg_endpoint}/v2/{project_id}/fgs/functions/{urn}/lts-log-detail" ;
        credentials,
        client
    )
}
