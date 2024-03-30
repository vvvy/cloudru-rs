
use clap::{Subcommand, Args};
use anyhow::Result;
use cloudru::{*, blocking::*};

#[derive(Args, Debug)]
struct LtsLogDetails {
    #[clap(short, long)]
    urn: String, 
}

#[derive(Subcommand, Debug)]
enum FgCommand {
    EnableLtsLogs,
    LtsLogDetails(LtsLogDetails),
}

#[derive(Args, Debug)]
pub struct Fg {
    #[clap(subcommand)]
    fg_command: FgCommand
}

pub fn handle_fg(client: fg::FgClient, fg: Fg) -> Result<JsonValue> {
    Ok(match fg.fg_command {
        FgCommand::EnableLtsLogs => {
            client.logging_to_lts_enable().cx("logging_to_lts_enable")
        }
        FgCommand::LtsLogDetails(LtsLogDetails{urn}) => {
            client.logging_to_lts_detail(&urn).cx("logging_to_lts_detail")
        }
    }?)
}
