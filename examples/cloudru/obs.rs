use clap::{Subcommand, Args};
use anyhow::{Result, anyhow};
use cloudru::{*, obs::*};
use crate::Config;

#[derive(Args, Debug)]
pub struct Obs {
    #[clap(short='e', long)]
    obs_endpoint: Option<String>,

    #[clap(subcommand)]
    obs_command: ObsCommand
}


#[derive(Subcommand, Debug)]
enum ObsCommand {
    Get(ObsGet),
    Put(ObsPut),
    Ls(ObsLs)
}

#[derive(Args, Debug)]
struct ObsGet {
    remote: String, 
    local: Option<String>, 
}

#[derive(Args, Debug)]
struct ObsPut {
    local: String,
    remote: String, 
}

#[derive(Args, Debug)]
struct ObsLs {
    remote: String, 
}

/// Returns bucket name and remaining path without leading '/'
fn split_bucket(remote: &str) -> (&str, &str) {
    let remote = remote.strip_prefix("https://").unwrap_or(remote);
    let remote = remote.strip_prefix("http://").unwrap_or(remote);
    let remote = remote.strip_prefix("/").unwrap_or(remote);
    remote.split_once('/').unwrap_or((remote, ""))
}

/// If target_path does not contain a file name (i.e. is empty or ends with '/'), extract the file name from source_path 
/// and append it to target_path, otherwise return target_path unchanged
/// 
/// Returns resulting target path
fn force_file_name(target_path: &str, source_path: &str) -> Result<String> {
    if source_path.ends_with('/') {
        return Err(anyhow!("source path must be a file"));
    }
    let file_name = source_path.rfind('/').map(|n| &source_path[n+1..]).unwrap_or(source_path);
    if target_path.ends_with('/'){ Ok(target_path.to_owned() + file_name)  }
    else if target_path.is_empty() { Ok(file_name.to_owned()) }
    else { Ok(target_path.to_owned())  }
}



#[test]
fn test_split_remote() {
    assert_eq!(split_bucket("bucket/file.name"), ("bucket", "file.name"));
    assert_eq!(split_bucket("bucket/path/to/file.name"), ("bucket", "path/to/file.name"));
    assert_eq!(split_bucket("bucket/"), ("bucket", ""));
    assert_eq!(split_bucket("bucket"), ("bucket", ""));
}

#[test]
fn test_force_file_name() {
    assert_eq!(force_file_name("a/b/c", "x/file.name").unwrap(), "a/b/c".to_owned());
    assert_eq!(force_file_name("a/b/c/", "x/file.name").unwrap(), "a/b/c/file.name".to_owned());
    assert_eq!(force_file_name("", "x/file.name").unwrap(), "file.name".to_owned());
    
    assert_eq!(force_file_name("a/b/c/", "file.name").unwrap(), "a/b/c/file.name".to_owned());
    assert_eq!(force_file_name("c", "file.name").unwrap(), "c".to_owned());
    assert_eq!(force_file_name("", "file.name").unwrap(), "file.name".to_owned());

}


pub fn handle_obs(aksk: AkSk, config: Config, obs: Obs) -> Result<JsonValue> {
    let endpoint: String = obs.obs_endpoint.or(config.endpoint.obs).ok_or(anyhow!("missing setting: obs_endpoint"))?;

    match obs.obs_command {
        ObsCommand::Get(get) => {
            let (bucket_name, source_path) = split_bucket(&get.remote);
            let target_path = force_file_name(
                get.local.as_ref().map(|x| x as &str).unwrap_or(""),
                source_path)?;
            let bucket = Bucket::new(bucket_name.to_owned(), endpoint, aksk)?;
            let mut output_file = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(&target_path)?;
            bucket.get_object(source_path, &mut output_file)?;
            Ok(JsonValue::Bool(true))
        }
        ObsCommand::Put(put) => {
            let (bucket_name, target_path) = split_bucket(&put.remote);
            let source_path = put.local;
            let target_path = force_file_name(target_path, &source_path)?;
            let bucket = Bucket::new(bucket_name.to_owned(), endpoint, aksk)?;
            let input_file = std::fs::File::open(&source_path)?;
            bucket.put_object(&target_path, input_file)?;
            Ok(JsonValue::Bool(true))
        }
        ObsCommand::Ls(ls) => {
            let (bucket_name, bucket_path) = split_bucket(&ls.remote);
            let bucket = Bucket::new(bucket_name.to_owned(), endpoint, aksk)?;
            bucket.list(Some(bucket_path))?;
            Ok(JsonValue::Bool(true))
        }
    }
    
}