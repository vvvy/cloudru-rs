use clap::{Subcommand, Args};
use anyhow::Result;
use cloudru::{*, blocking::*};

#[derive(Args, Debug)]
struct GetTables {
    /// Database name to query
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
    dli_command: DliCommand
}

pub fn handle_dli(client: dli::DliClient, dli: Dli) -> Result<JsonValue> {
    match dli.dli_command {
        DliCommand::GetDatabases => {
            println!("{:?}", client.get_databases()?);
        }
        DliCommand::GetTables(GetTables { database }) => {
            println!("{:?}", client.get_tables(&database)?);
        }
    }
    Ok(JsonValue::Bool(true))
}
