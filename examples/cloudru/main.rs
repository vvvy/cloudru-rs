mod apig;
mod fg;
mod obs;

use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use cloudru::*;

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
    #[clap(short='C', long, default_value=DEFAULT_CONFIG_FILE)]
    config_file: String,

    ///Full path to the credentials file to take ak/sk from
    #[clap(short='F', long, default_value=DEFAULT_CREDENTIALS_FILE)]
    credentials_file: String,

    ///Id of the credential to use (within the credentials file)
    #[clap(short='I', long, default_value=DEFAULT_CREDENTIAL)]
    credential_id: String,

    #[clap(short='L', long, default_value=DEFAULT_LOG_LEVEL)]
    level: Level
}


fn main() -> Result<()> {

    let args = Cli::parse();
    //println!("Hello, world: {:?}", args);
    let config = read_config(args.config_file)?;
    let aksk = read_credentials(args.credentials_file, args.credential_id)?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.level)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let rv = match args.command {
        Command::Apig(apig) => handle_apig(aksk, config, apig)?,
        Command::Fg(fg) => handle_fg(aksk, config, fg)?,
        Command::Obs(obs) => handle_obs(aksk, config, obs)?,
    };

    cloudru::json_to_writer_pretty(std::io::stdout(), &rv)?;
    Ok(())
}
