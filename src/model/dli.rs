use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default)]
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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug, Default)]
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

#[derive(Deserialize, Debug, Default)]
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
    pub is_success: Option<bool>,
    pub message: Option<String>,
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

#[derive(Deserialize, Debug, Default)]
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

#[derive(Deserialize, Debug)]
pub struct StorageProperty {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct Column {
    #[serde(rename = "column_name")]
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub is_partition_column: bool,
}
