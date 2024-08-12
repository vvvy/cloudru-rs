//use tracing::{debug, instrument};
//use reqwest::{header::{HeaderMap, HeaderValue}, Body, Method, Request, RequestBuilder};
//use url::Url;

use std::sync::Arc;

pub use crate::model::dli as model;
use super::*;
use crate::*;
use crate::config::svc_id;

pub struct DliClient {
    endpoint: String,
    project_id: String,
    credentials: Credentials,
    http_client: Arc<HttpClient>,
}

impl DliClient {
    pub fn new(endpoint: String, project_id: String, credentials: Credentials, http_client: Arc<HttpClient>) -> Self { 
        Self { endpoint, project_id, http_client, credentials } 
    }
    pub fn get_databases(&self)  -> Result<model::GetDatabasesResponse>  {
        let endpoint = &self.endpoint;
        let project_id  = &self.project_id;

        //GET https://dli.ru-moscow-1.hc.sbercloud.ru/v1.0/{{project_id}}/databases
        api_call!(GET /"{endpoint}/v1.0/{project_id}/databases" ;
            &self.credentials,
            &self.http_client
        )
    }

    pub fn get_tables(&self, database: &str)  -> Result<model::GetTablesResponse>  {
        let endpoint = &self.endpoint;
        let project_id  = &self.project_id;

        //GET https://dli.{ru-moscow-1.hc.sbercloud.ru/v1.0/{{project_id}}/databases/{{dli_database_name}}/tables
        api_call!(GET /"{endpoint}/v1.0/{project_id}/databases/{database}/tables" ;
            &self.credentials,
            &self.http_client
        )
    }
}

pub trait DliClientBuild {
    fn build_dli(&self) -> Result<DliClient>;
}

impl DliClientBuild for Client {
    fn build_dli(&self) -> Result<DliClient> {
        Ok(DliClient::new(
            self.resolve_endpoint(svc_id::dli)?,
            self.resolve_project_id()?,
            self.credentials.clone(),
            self.http_client.clone(),
        ))
    }
}
