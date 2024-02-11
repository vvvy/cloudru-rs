
use clap::{Subcommand, Args};
use anyhow::{Result, anyhow};
use cloudru::*;

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
    #[clap(short='e', long)]
    fg_endpoint: Option<String>,

    #[clap(short='p', long)]
    fg_project_id: Option<String>,

    #[clap(subcommand)]
    fg_command: FgCommand
}

pub fn handle_fg(aksk: AkSk, config: Config, fg: Fg) -> Result<JsonValue> {
    let endpoint = fg.fg_endpoint.or(config.endpoint.fg).ok_or(anyhow!("missing setting: fg_endpoint"))?;
    let project_id = fg.fg_project_id.or(config.project_id).ok_or(anyhow!("missing setting: fg_project_id"))?;

    Ok(match fg.fg_command {
        FgCommand::EnableLtsLogs => {
            cloudru::fg::logging_to_lts_enable(&endpoint, &project_id, &aksk).cx("logging_to_lts_enable")
        }
        FgCommand::LtsLogDetails(LtsLogDetails{urn}) => {
            cloudru::fg::logging_to_lts_detail(&endpoint, &project_id, &urn, &aksk).cx("logging_to_lts_detail")
        }
    }?)
}
