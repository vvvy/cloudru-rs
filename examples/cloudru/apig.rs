use std::fs::read_to_string;
use clap::{Subcommand, Args};
use anyhow::{Result, anyhow};

use cloudru::*;


#[derive(Args, Debug)]
struct ApigAddCertificate {
    #[clap(short, long)]
    group_id: String, 

    #[clap(short, long)]
    domain_id: String,

    #[clap(short='n', long)]
    cert_name: String,

    #[clap(short, long)]
    cert_content_file: String,

    #[clap(short, long)]
    private_key_file: String,
}

#[derive(Args, Debug)]
struct ApigGetCertificate {
    #[clap(short, long)]
    group_id: String, 

    #[clap(short, long)]
    domain_id: String,

    #[clap(short='i', long)]
    cert_id: String,
}

#[derive(Args, Debug)]
struct ApigDeleteCertificate {
    #[clap(short, long)]
    group_id: String, 

    #[clap(short, long)]
    domain_id: String,

    #[clap(short='i', long)]
    cert_id: String,
}


#[derive(Args, Debug)]
struct ApigGetGroupDetails {
    #[clap(short, long)]
    group_id: String, 
}

#[derive(Subcommand, Debug)]
enum ApigCommand {
    GetApiGroupDetails(ApigGetGroupDetails),
    AddCertificate(ApigAddCertificate),
    GetCertificate(ApigGetCertificate),
    DeleteCertificate(ApigDeleteCertificate)
}

#[derive(Args, Debug)]
pub struct Apig {
    #[clap(short='e', long)]
    apig_endpoint: Option<String>,

    #[clap(subcommand)]
    apig_command: ApigCommand
}


pub fn handle_apig(aksk: AkSk, config: Config, apig: Apig) -> Result<JsonValue> {
    let endpoint = apig.apig_endpoint.or(config.endpoint.apig).ok_or(anyhow!("missing setting: apig_endpoint"))?;

    match apig.apig_command {
        ApigCommand::GetApiGroupDetails(ApigGetGroupDetails{
            group_id
        }) => {
            let rv = cloudru::apig::get_api_group_detail(
                &endpoint,
                &group_id,  
                &aksk
            )?;

            Ok(rv)
        }
        ApigCommand::AddCertificate(ApigAddCertificate{
            group_id, domain_id, cert_name, cert_content_file, private_key_file
        }) => {
            //println!("SET CERT {group_id} {domain_id} {cert_name} {cert_content_file} {private_key_file}");
            let cert_content = read_to_string(cert_content_file)?;
            let private_key = read_to_string(private_key_file)?;
            let rv = cloudru::apig::add_certificate(
                &endpoint,
                &group_id, 
                &domain_id,
                &cert_name, 
                &cert_content, 
                &private_key, 
                &aksk
            )?;

            Ok(rv)
        }
        ApigCommand::GetCertificate(ApigGetCertificate{
            group_id, domain_id, cert_id
        }) => {
            let rv = cloudru::apig::get_certificate(
                &endpoint,
                &group_id, 
                &domain_id,
                &cert_id, 
                &aksk
            )?;

            Ok(rv)
        }
        ApigCommand::DeleteCertificate(ApigDeleteCertificate{
            group_id, domain_id, cert_id
        }) => {
            let rv = cloudru::apig::delete_certificate(
                &endpoint,
                &group_id, 
                &domain_id,
                &cert_id, 
                &aksk
            )?;

            Ok(rv)
        }
    }
}
