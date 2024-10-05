use clap::{Subcommand, Args};
use anyhow::{Result, anyhow};
use cloudru::{*, blocking::{*, obs::{ListObjectsContents, ListObjectsRequest}}};

#[derive(Args, Debug)]
pub struct Obs {
    #[clap(subcommand)]
    obs_command: ObsCommand
}

#[derive(Subcommand, Debug)]
enum ObsCommand {
    Get(ObsGet),
    Put(ObsPut),
    Ls(ObsLs),
    LsVersions(ObsLsVersions),
    BucketMetadata(ObsBucketMetadata),
    ObjectMetadata(ObsObjectMetadata),
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

    #[clap(long, short)]
    long: bool,

    #[clap(long, short)]
    marker: Option<String>,

    #[clap(long, short='n')]
    max_keys: Option<u32>,
}

#[derive(Args, Debug)]
struct ObsLsVersions {
    remote: String,

    #[clap(long, short)]
    long: bool,

    #[clap(long, short)]
    marker: Option<String>,

    #[clap(long, short='n')]
    max_keys: Option<u32>,
}

#[derive(Args, Debug)]
struct ObsBucketMetadata {
    bucket: String,
}

#[derive(Args, Debug)]
struct ObsObjectMetadata {
    remote: String, 
}

/// Returns bucket name and remaining path without leading '/'
fn split_bucket(remote: &str) -> (&str, &str) {
    let remote = remote.strip_prefix("https://").unwrap_or(remote);
    let remote = remote.strip_prefix("http://").unwrap_or(remote);
    let remote = remote.strip_prefix("obs://").unwrap_or(remote);
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


pub fn handle_obs(client: obs::ObsClient, obs: Obs) -> Result<JsonValue> {
    match obs.obs_command {
        ObsCommand::Get(get) => {
            let (bucket_name, source_path) = split_bucket(&get.remote);
            let target_path = force_file_name(
                get.local.as_ref().map(|x| x as &str).unwrap_or(""),
                source_path)?;
            let bucket = client.bucket(bucket_name.to_owned())?;
            let mut output_file = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(&target_path)?;
            bucket.get_object(source_path, &mut output_file)?;
            Ok(JsonValue::Bool(true))
        }
        ObsCommand::Put(put) => {
            let (bucket_name, target_path) = split_bucket(&put.remote);
            let source_path = put.local;
            let target_path = force_file_name(target_path, &source_path)?;
            let bucket = client.bucket(bucket_name.to_owned())?;
            let input_file = std::fs::File::open(&source_path)?;
            bucket.put_object(&target_path, input_file)?;
            Ok(JsonValue::Bool(true))
        }
        ObsCommand::Ls(ls) => {
            let (bucket_name, bucket_path) = split_bucket(&ls.remote);
            let bucket = client.bucket(bucket_name.to_owned())?;

            let list_request = ListObjectsRequest {
                prefix: Some(bucket_path),
                marker: ls.marker.as_deref(),
                max_keys: ls.max_keys,
                ..Default::default()
            };
            let list = bucket.list_objects(list_request)?;
            let Some(contents) = list.contents else { return Ok(JsonValue::Bool(true)) };
            for ListObjectsContents { 
                key, 
                last_modified, 
                etag, 
                size, 
                storage_class, 
                type_, 
                owner,
                ..
         } in contents {
                let type_ = type_.as_deref().unwrap_or("-");
                let owner = owner.as_ref().map(|s| &s.id as &str).unwrap_or("-");
                if ls.long {
                    println!("{etag}\t{storage_class}\t{owner}\t{type_}\t{size}\t{last_modified}\t{key}")
                } else {
                    println!("{type_}\t{size}\t{last_modified}\t{key}")
                }
            }
            if let Some(next_marker) = list.next_marker {
                println!("next_marker: {next_marker}")
            }
            Ok(JsonValue::Bool(true))
        }
        ObsCommand::LsVersions(ls) => {
            let (bucket_name, bucket_path) = split_bucket(&ls.remote);
            let bucket = client.bucket(bucket_name.to_owned())?;

            let list_request = ListObjectsRequest {
                prefix: Some(bucket_path),
                marker: ls.marker.as_deref(),
                max_keys: ls.max_keys,
                ..Default::default()
            };
            let list = bucket.list_object_versions(list_request)?;
            let Some(contents) = list.version else { return Ok(JsonValue::Bool(true)) };
            for ListObjectsContents { 
                key, 
                last_modified, 
                etag, 
                size, 
                storage_class, 
                type_, 
                owner,
                version_id,
                is_latest,
         } in contents {
                let type_ = type_.as_deref().unwrap_or("-");
                let owner = owner.as_ref().map(|s| &s.id as &str).unwrap_or("-");
                let version_id = version_id.unwrap();
                let is_latest = is_latest.unwrap();
                if ls.long {
                    println!("{version_id}\t{is_latest}\t{etag}\t{storage_class}\t{owner}\t{type_}\t{size}\t{last_modified}\t{key}")
                } else {
                    println!("{version_id}\t{is_latest}\t{type_}\t{size}\t{last_modified}\t{key}")
                }
            }
            if let Some(next_key_marker) = list.next_key_marker {
                println!("next_key_marker: {next_key_marker}")
            }
            Ok(JsonValue::Bool(true))
        }
        ObsCommand::BucketMetadata(bmd) => {
            let bucket = client.bucket(bmd.bucket)?;
            let md = bucket.get_bucket_meta()?;
            println!("{md:?}");
            Ok(JsonValue::Bool(true))
        }

        ObsCommand::ObjectMetadata(omd) => {
            let (bucket_name, source_path) = split_bucket(&omd.remote);
            let bucket = client.bucket(bucket_name.to_owned())?;
            let md = bucket.get_object_meta(source_path)?;
            println!("{md:?}");
            Ok(JsonValue::Bool(true))
        }
    }
    
}