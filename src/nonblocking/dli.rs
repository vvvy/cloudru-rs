//use tracing::{debug, instrument};
//use reqwest::{header::{HeaderMap, HeaderValue}, Body, Method, Request, RequestBuilder};
//use url::Url;

pub use crate::model::dli as model;
use super::*;
use crate::*;

pub struct DliClient {
    endpoint: String,
    project_id: String,
    credentials: Credentials,
    http_client: HttpClient,
}

impl DliClient {
    pub fn new(endpoint: String, project_id: String, credentials: Credentials, http_client: HttpClient) -> Self { 
        Self { endpoint, project_id, http_client, credentials } 
    }
    pub async fn get_databases(&self)  -> Result<model::GetDatabasesResponse>  {
        let endpoint = &self.endpoint;
        let project_id  = &self.project_id;

        //GET https://dli.ru-moscow-1.hc.sbercloud.ru/v1.0/{{project_id}}/databases
        api_call!(GET /"{endpoint}/v1.0/{project_id}/databases" ;
            &self.credentials,
            &self.http_client
        )
    }
    pub async fn get_tables(&self, database: &str)  -> Result<model::GetTablesResponse>  {
        let endpoint = &self.endpoint;
        let project_id  = &self.project_id;

        //GET https://dli.{ru-moscow-1.hc.sbercloud.ru/v1.0/{{project_id}}/databases/{{dli_database_name}}/tables
        api_call!(GET /"{endpoint}/v1.0/{project_id}/databases/{database}/tables" ;
            &self.credentials,
            &self.http_client
        )
    }
}
