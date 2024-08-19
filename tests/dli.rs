use anyhow::Result;
use blocking::dli::{DliClient, DliClientBuild};
use clap::{Args, Subcommand};
use cloudru::{blocking::client::*, *};

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
    Ok(())
}

#[test]
fn test_get_tables() -> Result<()> {
    let dli_client = create_dli_client()?;
    let database = "dm_top100".to_string();
    let response = dli_client.get_tables(&database)?;
    println!("get_tables response: {:?}", response);
    Ok(())
}

#[test]
fn test_get_partitions() -> Result<()> {
    let dli_client = create_dli_client()?;
    let db_name = "dm_top100".to_string();
    let table_name = "sessions_eb".to_string();
    let response = dli_client.get_partitions(&db_name, &table_name, None, None, None)?;
    println!("get_partitions response: {:?}", response);
    Ok(())
}
