mod apig;
mod fg;
mod obs;

use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use cloudru::{*, blocking::*};

use config::*;
use apig::*;
use fg::*;
use obs::*;

#[derive(Subcommand, Debug)]
enum Command {
    Apig(Apig),
    Fg(Fg),
    Obs(Obs),
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    ///Full path to the config file
    #[clap(short='C', long)]
    config_file: Option<String>,

    ///Full path to the credentials file to take ak/sk from
    #[clap(short='F', long)]
    credentials_file: Option<String>,

    ///Id of the credential to use (within the credentials file)
    #[clap(short='I', long)]
    credential_id: Option<String>,

    ///Project id
    #[clap(short='P', long)]
    project_id: Option<String>,

    ///Region id
    #[clap(short='R', long)]
    region: Option<String>,

    #[clap(short='L', long, default_value=DEFAULT_LOG_LEVEL)]
    level: Level
}


fn main() -> Result<()> {

    let args = Cli::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.level)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let client_builder: ClientBuilder = ClientBuilder { 
        config_file: args.config_file,
        credentials_file: args.credentials_file,
        credentials_id: args.credential_id,
        project_id: args.project_id,
        region: args.region,
        ..ClientBuilder::new() };

    let client = client_builder.build()?;

    let rv = match args.command {
        Command::Apig(apig) => handle_apig(client.apig()?, apig)?,
        Command::Fg(fg) => handle_fg(client.fg()?, fg)?,
        Command::Obs(obs) => handle_obs(client.obs()?, obs)?,
    };

    cloudru::json_to_writer_pretty(std::io::stdout(), &rv)?;
    Ok(())
}
