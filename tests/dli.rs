use std::{fs::File, io::Write};

use anyhow::Result;
use blocking::dli::{DliClient, DliClientBuild};
use clap::{Args, Subcommand};
use cloudru::{blocking::client::*, *};
use serde_json::to_string_pretty;

#[derive(Args, Debug)]
struct GetTables {
    database: String,
}

#[derive(Subcommand, Debug)]
enum DliCommand {
    GetDatabases,
    GetTables(GetTables),
}

#[derive(Args, Debug)]
pub struct Dli {
    #[clap(subcommand)]
    dli_command: DliCommand,
}

fn create_dli_client() -> Result<DliClient> {
    let client = Client::builder()
        .from_environment(Some("CLOUDRU"), Some("DATA"))
        .build()?;
    let dli_client = client.build_dli()?;
    Ok(dli_client)
}

#[test]
fn test_get_databases() -> Result<()> {
    let dli_client = create_dli_client()?;
    let response = dli_client.get_databases()?;
    println!("get_databases response: {:?}", response);

    // todo: write macro
    let json_string = to_string_pretty(&response).unwrap();
    let mut file = File::create("databases_response.json").unwrap();
    file.write_all(json_string.as_bytes()).unwrap();

    Ok(())
}

#[test]
fn test_get_tables() -> Result<()> {
    let dli_client = create_dli_client()?;
    let database = "ods_sber".to_string();
    let response = dli_client.get_tables(&database)?;
    println!("get_tables response: {:?}", response);

    let json_string = to_string_pretty(&response).unwrap();
    let mut file = File::create(format!("tables_{}_response.json", database)).unwrap();
    file.write_all(json_string.as_bytes()).unwrap();

    Ok(())
}

#[test]
fn test_get_table() -> Result<()> {
    let dli_client = create_dli_client()?;
    let database = "ods_sber".to_string();
    let table_name = "dbo_clients".to_string();
    let response = dli_client.get_table(&database, &table_name)?;

    let json_string = to_string_pretty(&response).unwrap();
    let mut file = File::create(format!("table_{}_response.json", table_name)).unwrap();
    file.write_all(json_string.as_bytes()).unwrap();

    println!("get_table response: {:?}", response);

    Ok(())
}

#[test]
fn test_get_partitions() -> Result<()> {
    let dli_client = create_dli_client()?;
    let db_name = "dm_top100".to_string();
    let table_name = "sessions".to_string();
    let response = dli_client.get_partitions(&db_name, &table_name, Some(100), Some(0), None)?;
    
    let json_string = to_string_pretty(&response).unwrap();
    let mut file = File::create(format!("partitions_{}_response.json", table_name)).unwrap();
    file.write_all(json_string.as_bytes()).unwrap();

    println!("get_partitions response: {:?}", response);
    Ok(())
}
