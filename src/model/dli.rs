use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetDatabasesResponse {
    /// is_success
    ///
    /// Indicates whether the request is successfully executed. Value true indicates that the request is successfully executed.
    ///
    /// Requred:No. Type: Boolean
    pub is_success: Option<bool>,

    /// message
    ///
    /// System prompt. If execution succeeds, the parameter setting may be left blank.
    /// Requred: No. Type: String
    pub message: Option<String>,

    /// database_count
    ///
    /// Total number of databases.
    ///
    /// Requred: No. Type: Integer
    pub database_count: Option<i64>,

    /// databases
    ///
    /// Database information.
    ///
    /// Requred: No. Type: Array of objects
    pub databases: Option<Vec<Database>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    /// database_name
    ///
    /// Name of a database.
    ///
    /// Requred: No. Type: String
    pub database_name: Option<String>,

    /// owner
    ///
    /// Creator of a database.
    ///
    /// Requred: No. Type: String
    pub owner: Option<String>,

    /// table_number
    ///
    /// Number of tables in a database.
    ///
    /// Requred: No. Type: Integer
    pub table_number: Option<i64>,

    /// description
    ///
    /// Information about a database.
    ///
    /// Requred: No. Type: String
    pub description: Option<String>,

    /// enterprise_project_id
    ///
    /// Enterprise project ID. The value 0 indicates the default enterprise project.
    ///
    /// Requred: Yes. Type: String
    pub enterprise_project_id: String,
}

#[derive(Debug, Default)]
pub struct GetTablesRequest {
    /// keyword
    ///
    /// Keywords used to filter table names.
    ///
    /// Mandatory: No. Type: String
    pub keyword: Option<String>,

    /// with-detail
    ///
    /// Whether to obtain detailed information about tables (such as owner and size). The default value is false.
    ///
    /// Mandatory: No. Type: Boolean
    pub with_detail: Option<bool>,

    /// page-size
    ///
    /// Paging size. The minimum value is 1 and the maximum value is 100.
    ///
    /// Mandatory: No. Type: Integer
    pub page_size: Option<i64>,

    /// current-page
    ///
    /// Current page number. The minimum value is 1.
    ///
    /// Mandatory: No. Type: Integer
    pub current_page: Option<i64>,

    /// with-priv
    ///
    /// Whether to return permission information.
    ///
    /// Mandatory: No. Type: Boolean
    pub with_priv: Option<bool>,

    /// table-type
    ///
    /// Table type. The options are as follows: `MANAGED_TABLE` (DLI table), `EXTERNAL_TABLE` (OBS table), `VIRTUAL_VIEW` (view)
    ///
    /// Mandatory: No. Type: String
    pub table_type: Option<String>,

    /// datasource-type
    ///
    /// Data source type. The options are as follows: `CloudTable`, `CSS`, `DLI`, `DWS`, `Geomesa`, `HBase`, `JDBC`, `Mongo`, `OBS`,
    /// `ODPS`, `OpenTSDB`, `Redis`, and `RDS`
    ///
    /// Mandatory: No. Type: String
    pub datasource_type: Option<String>,

    /// without-tablemeta
    ///
    /// Whether to obtain the metadata of a table. The default value is false. If this parameter is set to true, the response speed can be greatly improved.
    ///
    /// Mandatory: No. Type: Boolean
    pub without_tablemeta: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetTablesResponse {
    /// is_success
    ///
    /// Indicates whether the request is successfully executed. Value true indicates that the request is successfully executed.
    ///
    /// Mandatory: Yes. Type: Boolean
    pub is_success: Option<bool>,

    /// message
    ///
    /// System prompt. If execution succeeds, the parameter setting may be left blank.
    /// Mandatory: Yes. Type: String
    pub message: Option<String>,

    /// table_count
    ///
    /// Total number of tables.
    ///
    /// Mandatory: Yes. Type: Integer
    pub table_count: Option<i64>,

    /// tables
    ///
    /// Table information.
    ///
    /// Mandatory: Yes. Type: Array of objects
    pub tables: Option<Vec<Table>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Table {
    /// create_time
    ///
    /// Time when a table is created. The timestamp is expressed in milliseconds.
    ///
    /// Mandatory: Yes. Type: Long
    //#[serde(rename="create_time")]
    pub create_time: Option<i64>,

    /// data_type
    ///
    /// Type of the data to be added to the OBS table. The options are as follows: Parquet, ORC, CSV, JSON, and Avro.
    /// NOTE: This parameter is available only for OBS tables.
    ///
    /// Mandatory: No. Type: String
    pub data_type: Option<String>,

    /// data_location
    ///
    /// Data storage location, which can be DLI or OBS.
    ///
    /// Mandatory: Yes. Type: String
    pub data_location: String,

    /// last_access_time
    ///
    /// Time when a table is last updated. The timestamp is expressed in milliseconds.
    ///
    /// Mandatory: Yes. Type: Long
    pub last_access_time: Option<i64>,

    /// location
    ///
    /// Storage path on the OBS table. NOTE: This parameter is available only for OBS tables.
    ///
    /// Mandatory: No. Type: String
    pub location: Option<String>,

    /// owner
    ///
    /// Table owner.
    ///
    /// Mandatory: Yes. Type: String
    pub owner: Option<String>,

    /// table_name
    ///
    /// Name of a table.
    ///
    /// Mandatory: Yes. Type: String
    pub table_name: String,

    /// table_size
    ///
    /// Size of a DLI table. Set parameter to 0 for non-DLI tables. The unit is byte.
    ///
    /// Mandatory: Yes. Type: Long
    pub table_size: Option<i64>,

    /// table_type
    ///
    /// Type of a table: `EXTERNAL``: Indicates an OBS table. `MANAGED`: Indicates a DLI table. `VIEW``: Indicates a view
    ///
    /// Mandatory: Yes. Type: String
    pub table_type: String,

    /// partition_columns
    ///
    /// Partition field. This parameter is valid only for OBS partition tables.
    ///
    /// Mandatory: No. Type: String
    pub partition_columns: Option<String>,

    /// page-size
    ///
    /// Paging size. The minimum value is 1 and the maximum value is 100.
    ///
    /// Mandatory: No. Type: Integer
    #[serde(rename = "page-size")]
    pub page_size: Option<i64>,

    /// current-page
    ///
    /// Current page number. The minimum value is 1.
    ///
    /// Mandatory: No. Type: Integer
    #[serde(rename = "current-page")]
    pub current_page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GetPartitionsResponse {
    pub is_success: bool,
    pub message: String,
    pub partitions: Option<Partitions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Partitions {
    pub total_count: i64,
    pub partition_infos: Vec<PartitionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub partition_name: String,
    pub create_time: i64,
    pub last_access_time: i64,
    pub locations: Option<Vec<String>>,
    pub last_ddl_time: Option<i64>,
    pub num_rows: Option<i64>,
    pub num_files: Option<i64>,
    pub total_size: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetTableResponse {
    pub is_success: bool,
    pub message: String,
    pub column_count: u32,
    pub columns: Vec<Column>,
    pub table_type: String,
    pub data_type: Option<String>,
    pub data_location: Option<String>,
    pub storage_properties: Option<Vec<StorageProperty>>,
    pub table_comment: Option<String>,
    pub create_table_sql: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StorageProperty {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Column {
    #[serde(rename = "column_name")]
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub is_partition_column: bool,
}

/// Represents the response from the DLI Submit SQL Job API.
///
/// This struct corresponds to the API response documented here:
/// API Documentation: [Submitting a SQL Job](https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0102.html)
#[derive(Debug, Deserialize)]
pub struct SubmitSqlJobResponse {
    /// Indicates whether the request was successfully sent.
    ///
    /// - **Mandatory**: Yes
    /// - **Type**: Boolean
    /// - **Description**: Value `true` indicates that the request is successfully sent.
    pub is_success: bool,

    /// System prompt message.
    ///
    /// - **Mandatory**: Yes
    /// - **Type**: String (optional)
    /// - **Description**: If execution succeeds, this field may be empty.
    pub message: Option<String>,

    /// ID of the submitted job.
    ///
    /// - **Mandatory**: Yes
    /// - **Type**: String (optional)
    /// - **Description**: The job ID can be used to query the job status and results.
    pub job_id: Option<String>,

    /// Type of the submitted job.
    ///
    /// - **Mandatory**: Yes
    /// - **Type**: String (optional)
    /// - **Description**: Job type. Possible values:
    ///   - `DDL`
    ///   - `DCL`
    ///   - `IMPORT`
    ///   - `EXPORT`
    ///   - `QUERY`
    ///   - `INSERT`
    pub job_type: Option<String>,

    /// Job execution mode.
    ///
    /// - **Mandatory**: No
    /// - **Type**: String (optional)
    /// - **Description**: The options are:
    ///   - `async`: Asynchronous
    ///   - `sync`: Synchronous
    pub job_mode: Option<String>,
}

impl Default for SubmitSqlJobResponse {
    fn default() -> Self {
        SubmitSqlJobResponse {
            is_success: false,
            message: None,
            job_id: None,
            job_type: None,
            job_mode: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// Represents the response from the Query Job Status API.
///
/// This struct contains detailed information about the status of a job submitted
/// to the DLI service.
///
/// # API Reference
///
/// API Documentation: [Querying Job Status](https://support.hc.sbercloud.ru/en-us/api/dli/dli_02_0021.html)
#[derive(Debug, Deserialize)]
pub struct QueryJobStatusResponse {
    /// Indicates whether the request was successfully executed.
    ///
    /// - **Type**: Boolean
    /// - **Description**: `true` if the request was successful.
    pub is_success: bool,

    /// System prompt message.
    ///
    /// - **Type**: String (optional)
    /// - **Description**: If execution succeeds, this field may be empty.
    pub message: Option<String>,

    /// Job ID of the queried job.
    ///
    /// - **Type**: String (optional)
    /// - **Description**: The ID of the job being queried.
    pub job_id: Option<String>,

    /// The type of the job.
    ///
    /// - **Type**: String (optional)
    /// - **Description**: Possible values include:
    ///   - `DDL`
    ///   - `DCL`
    ///   - `IMPORT`
    ///   - `EXPORT`
    ///   - `QUERY`
    ///   - `INSERT`
    ///   - `DATA_MIGRATION`
    ///   - `UPDATE`
    ///   - `DELETE`
    ///   - `RESTART_QUEUE`
    ///   - `SCALE_QUEUE`
    pub job_type: Option<String>,

    /// The execution mode of the job.
    ///
    /// - **Type**: String (optional)
    /// - **Description**: Can be either:
    ///   - `async`: Asynchronous mode
    ///   - `sync`: Synchronous mode
    pub job_mode: Option<String>,

    /// Name of the queue where the job was submitted.
    pub queue_name: Option<String>,

    /// User who submitted the job.
    pub owner: Option<String>,

    /// Start time of the job (in milliseconds since the epoch).
    pub start_time: Option<u64>,

    /// Duration of the job execution (in milliseconds).
    pub duration: Option<u64>,

    /// Current status of the job.
    ///
    /// Possible values include:
    /// - `RUNNING`
    /// - `SCALING`
    /// - `LAUNCHING`
    /// - `FINISHED`
    /// - `FAILED`
    /// - `CANCELLED`
    pub status: Option<String>,

    /// Number of rows scanned during the Insert job execution.
    pub input_row_count: Option<u64>,

    /// Number of bad rows encountered during the Insert job execution.
    pub bad_row_count: Option<u64>,

    /// Size of input data scanned during the job execution (in bytes).
    pub input_size: Option<u64>,

    /// Total number of records returned or inserted by the job.
    pub result_count: Option<u32>,

    /// Name of the database associated with the job.
    ///
    /// This field is only valid for jobs of type `IMPORT`, `EXPORT`, or `QUERY`.
    pub database_name: Option<String>,

    /// Name of the table associated with the job.
    ///
    /// This field is only valid for jobs of type `IMPORT`, `EXPORT`, or `QUERY`.
    pub table_name: Option<String>,

    /// Additional details about the job, typically as a JSON string.
    pub detail: Option<String>,

    /// The SQL statement used in the job.
    pub statement: Option<String>,

    /// Tags associated with the job.
    pub tags: Option<Vec<Tag>>,

    /// User-defined configuration details as a JSON string.
    pub user_conf: Option<String>,

    /// The storage format of job results.
    ///
    /// Currently, only `CSV` is supported.
    pub result_format: Option<String>,

    /// Path to the job results in Object Storage Service (OBS).
    pub result_path: Option<String>,
}

impl Default for QueryJobStatusResponse {
    fn default() -> Self {
        QueryJobStatusResponse {
            is_success: false,
            message: None,
            job_id: None,
            job_type: None,
            job_mode: None,
            queue_name: None,
            owner: None,
            start_time: None,
            duration: None,
            status: None,
            input_row_count: None,
            bad_row_count: None,
            input_size: None,
            result_count: None,
            database_name: None,
            table_name: None,
            detail: None,
            statement: None,
            tags: None,
            user_conf: None,
            result_format: None,
            result_path: None,
        }
    }
}
