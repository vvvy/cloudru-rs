//use tracing::{debug, instrument};
//use reqwest::{header::{HeaderMap, HeaderValue}, Body, Method, Request, RequestBuilder};
//use url::Url;

use serde_json::json;

use super::*;
pub use crate::model::dli as model;
use crate::*;

pub struct DliClient {
    endpoint: String,
    project_id: String,
    credentials: Credentials,
    http_client: HttpClient,
}

impl DliClient {
    pub fn new(
        endpoint: String,
        project_id: String,
        credentials: Credentials,
        http_client: HttpClient,
    ) -> Self {
        Self {
            endpoint,
            project_id,
            http_client,
            credentials,
        }
    }
    pub async fn get_databases(&self) -> Result<model::GetDatabasesResponse> {
        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        //GET https://dli.ru-moscow-1.hc.sbercloud.ru/v1.0/{{project_id}}/databases
        api_call!(GET /"{endpoint}/v1.0/{project_id}/databases" ;
            &self.credentials,
            &self.http_client
        )
    }
    pub async fn get_tables(&self, database: &str) -> Result<model::GetTablesResponse> {
        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        //GET https://dli.{ru-moscow-1.hc.sbercloud.ru/v1.0/{{project_id}}/databases/{{dli_database_name}}/tables
        api_call!(GET /"{endpoint}/v1.0/{project_id}/databases/{database}/tables" ;
            &self.credentials,
            &self.http_client
        )
    }

    // https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0102.html
    pub async fn submit_sql_job(
        &self,
        sql: &str,
        currentdb: Option<&str>,
        queue_name: Option<&str>,
        conf: Option<Vec<String>>,
        tags: Option<Vec<model::Tag>>,
    ) -> Result<model::SubmitSqlJobResponse> {
        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        let url = format!("{}/v1.0/{}/jobs/submit-job", endpoint, project_id);

        let request_body = json!({
            "sql": sql,
            "currentdb": currentdb.unwrap_or(""),
            "queue_name": queue_name.unwrap_or("default"),
            "conf": conf.unwrap_or_default(),
            "tags": tags.unwrap_or_default(),
        });

        api_call!(POST /"{url}" ;
            &request_body,
            &self.credentials,
            &self.http_client
        )
    }

    // https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0021.html
    pub async fn query_job_status(&self, job_id: &str) -> Result<model::QueryJobStatusResponse> {
        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        let url = format!("{}/v1.0/{}/jobs/{}/status", endpoint, project_id, job_id);

        api_call!(GET /"{url}" ;
            &self.credentials,
            &self.http_client
        )
    }
}
