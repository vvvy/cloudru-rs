//use tracing::{debug, instrument};
//use reqwest::{header::{HeaderMap, HeaderValue}, Body, Method, Request, RequestBuilder};
//use url::Url;

use std::sync::Arc;

use serde_json::json;

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
            query_params.push(filter.to_owned());
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

    // https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0033.html
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

    /// This method submits a SQL job to a specified queue, allowing the execution of various SQL statements,
    /// including DDL, DCL, IMPORT, QUERY, and INSERT. It enables fine-grained configuration and tagging
    /// for submitted jobs.
    ///
    /// # Arguments
    ///
    /// - `sql`: A mandatory SQL statement to execute.
    /// - `currentdb`: Optional. Specifies the database where the SQL statement will be executed.
    ///   Defaults to an empty string if not provided.
    /// - `queue_name`: Optional. The name of the queue to which the job will be submitted.
    ///   Defaults to "default" if not specified. The queue name must consist of letters, digits,
    ///   and underscores but cannot contain only digits or start with an underscore.
    /// - `conf`: Optional. A vector of configuration strings in `key=value` format. These configurations
    ///   allow fine-tuning of the job execution environment (e.g., setting partition numbers).
    /// - `tags`: Optional. A vector of `Tag` objects used for labeling the job. Each tag includes a
    ///   key-value pair (e.g., workspace and job name).
    ///
    /// # API Documentation
    ///
    /// Refer to the [Submit SQL Job API](https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0102.html)
    /// for detailed documentation on this API.
    ///
    /// # Returns
    ///
    /// - `Ok(SubmitSqlJobResponse)`: Contains details of the submitted job, including the job ID and type.
    /// - `Err`: An error object if the API call fails or the response cannot be parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let sql = "ALTER TABLE your_table_name ADD PARTITION (dt='2024-11-25') LOCATION 'obs://data/sales/dt=2024-11-25'";
    /// let currentdb = Some("db1");
    /// let queue_name = Some("default");
    /// let conf = Some(vec!["spark.sql.shuffle.partitions=200".to_string()]);
    /// let tags = Some(vec![
    ///     model::Tag { key: "workspace".to_string(), value: "space1".to_string() },
    ///     model::Tag { key: "jobName".to_string(), value: "name1".to_string() },
    /// ]);
    ///
    /// match dli_client.submit_sql_job(sql, currentdb, queue_name, conf, tags) {
    ///     Ok(response) => {
    ///         println!("Job submitted successfully. Job ID: {}", response.job_id);
    ///         println!("Job Type: {}", response.job_type);
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Failed to submit SQL job: {:?}", err);
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - If `tags` or `conf` are not specified, defaults are applied.
    /// - For further job management, see related APIs such as querying job status, details, and execution results.
    pub fn submit_sql_job(
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

    /// Queries the status of a submitted job.
    ///
    /// This method retrieves detailed information about the status of a job submitted via
    /// the DLI SQL Job API.
    ///
    /// # Arguments
    ///
    /// - `job_id`: The ID of the job whose status is being queried. This is mandatory and must
    ///   match the job ID returned when the job was initially submitted.
    ///
    /// # API Documentation
    ///
    /// Refer to the [Querying Job Status API](https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0021.html)
    /// for more details.
    ///
    /// # Returns
    ///
    /// - `Ok(QueryJobStatusResponse)`: Contains the job status details, such as job type, execution mode, start time, duration, and result details if the query succeeds.
    /// - `Err`: An error object if the API call fails or the response cannot be parsed.
    ///
    /// # Response Fields
    ///
    /// The method returns the following key details (refer to API documentation for full response details):
    /// - `is_success` (Boolean): Whether the request was successfully executed.
    /// - `job_id` (String): The unique identifier of the job.
    /// - `job_type` (String): The type of the job (e.g., QUERY, INSERT).
    /// - `status` (String): The current status of the job (e.g., RUNNING, FINISHED, FAILED).
    /// - `result_count` (Integer): Total number of records returned or inserted by the job.
    /// - `start_time` (Long): Timestamp when the job started.
    /// - `duration` (Long): Job execution duration in milliseconds.
    /// - `result_path` (String): OBS path of the job results (if applicable).
    ///
    /// # Example
    ///
    /// ```no_run
    /// let job_id = "208b08d4-0dc2-4dd7-8879-ddd4c020d7aa";
    ///
    /// match dli_client.query_job_status(job_id) {
    ///     Ok(response) => {
    ///         println!("Job Status: {}", response.status);
    ///         println!("Result Count: {}", response.result_count);
    ///         println!("Execution Duration: {} ms", response.duration);
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Failed to query job status: {:?}", err);
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - The `job_id` is mandatory and is used to identify the specific job whose status is being queried.
    /// - The API returns comprehensive details about the job, including SQL statements and job tags.
    /// - For additional information about job results and details, see the DLI SQL Job-related APIs.
    pub fn query_job_status(&self, job_id: &str) -> Result<model::QueryJobStatusResponse> {
        let endpoint = &self.endpoint;
        let project_id = &self.project_id;

        // Build the URL for the API call
        let url = format!("{}/v1.0/{}/jobs/{}/status", endpoint, project_id, job_id);

        // Perform the API call
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
