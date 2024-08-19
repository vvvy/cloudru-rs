//use tracing::{debug, instrument};
//use reqwest::{header::{HeaderMap, HeaderValue}, Body, Method, Request, RequestBuilder};
//use url::Url;

use std::sync::Arc;

use super::*;
use crate::config::svc_id;
pub use crate::model::dli as model;
use crate::*;

pub struct DliClient {
    endpoint: String,
    project_id: String,
    credentials: Credentials,
    http_client: Arc<HttpClient>,
}

impl DliClient {
    pub fn new(
        endpoint: String,
        project_id: String,
        credentials: Credentials,
        http_client: Arc<HttpClient>,
    ) -> Self {
        Self {
            endpoint,
            project_id,
            http_client,
            credentials,
        }
    }

    // api doc - https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0029.html
    pub fn get_databases(&self) -> Result<model::GetDatabasesResponse> {
        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        //GET https://dli.ru-moscow-1.hc.sbercloud.ru/v1.0/{{project_id}}/databases
        api_call!(GET /"{endpoint}/v1.0/{project_id}/databases" ;
            &self.credentials,
            &self.http_client
        )
    }


    // api doc - https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0105.html
    pub fn get_tables(&self, database: &str) -> Result<model::GetTablesResponse> {
        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        //GET https://dli.{ru-moscow-1.hc.sbercloud.ru/v1.0/{{project_id}}/databases/{{dli_database_name}}/tables
        api_call!(GET /"{endpoint}/v1.0/{project_id}/databases/{database}/tables" ;
            &self.credentials,
            &self.http_client
        )
    }

    // api doc - https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0250.html
    pub fn get_partitions(
        &self,
        database_name: &str,
        table_name: &str,
        limit: Option<i32>,
        offset: Option<i32>,
        filter: Option<&str>,
    ) -> Result<model::GetPartitionsResponse> {
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let filter = filter.unwrap_or("");

        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        let mut url = format!(
            "{}/v1.0/{}/databases/{}/tables/{}/partitions",
            endpoint, project_id, database_name, table_name
        );

        let mut query_params = vec![];
        query_params.push(format!("limit={}", limit));
        query_params.push(format!("offset={}", offset));
        if !filter.is_empty() {
            query_params.push(format!("filter={}", filter));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        api_call!(GET /"{url}" ;
            &self.credentials,
            &self.http_client
        )
    }

    pub fn get_table(
        &self,
        database_name: &str,
        table_name: &str,
    ) -> Result<model::GetTableResponse> {
        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        let url = format!(
            "{}/v1.0/{}/databases/{}/tables/{}",
            endpoint, project_id, database_name, table_name
        );

        api_call!(GET /"{url}" ;
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
