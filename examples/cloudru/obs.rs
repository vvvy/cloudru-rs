use std::{collections::BTreeMap, fmt, io::Write};

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
    PutStr(ObsPutStr),
    Ls(ObsLs),
    LsVersions(ObsLsVersions),
    Du(ObsDu),
    BucketMetadata(ObsBucketMetadata),
    ObjectMetadata(ObsObjectMetadata),
}

#[derive(Args, Debug)]
struct ObsGet {
    remote: String, 
    local: Option<String>,

    #[clap(long, short)]
    version_id: Option<String>,
}

#[derive(Args, Debug)]
struct ObsPut {
    local: String,
    remote: String, 
}

#[derive(Args, Debug)]
struct ObsPutStr {
    string: String,
    remote: String, 
}


#[derive(Args, Debug)]
struct ObsLs {
    remote: String,

    /// Long listing
    #[clap(long, short)]
    long: bool,

    /// Marker to start at
    #[clap(long, short)]
    marker: Option<String>,

    /// Max keys per page
    #[clap(long, short='n')]
    max_keys: Option<u32>,

    /// List this number of pages
    #[clap(long, short='c')]
    pages: Option<usize>,

    /// List all objects under the prefix
    #[clap(long, short='a')]
    all: bool,    

    /// Print non-file entries
    #[clap(long, short='r')]
    raw: bool,
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
struct ObsDu {
    remote: String,

    /// Depth
    #[clap(long, short, default_value_t=1)]
    depth: usize,

    /// Marker to start at
    #[clap(long, short)]
    marker: Option<String>,

    /// Max keys per page
    #[clap(long, short='n')]
    max_keys: Option<u32>,

    /// Quit after this number of pages
    #[clap(long, short='c')]
    pages: Option<usize>,
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

struct HrSize(u64);

impl fmt::Display for HrSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = self.0;
        if f.alternate() {
            return w.fmt(f);
        }
        if w < 1024 {
            w.fmt(f)?;
            return write!(f, " ");
        }
        let w = w/1024;
        if w < 1024 {
            w.fmt(f)?;
            return write!(f, "K");
        }
        let w = w/1024;
        if w < 1024 {
            w.fmt(f)?;
            return write!(f, "M");
        }
        let w = w/1024;
        if w < 1024 {
            w.fmt(f)?;
            return write!(f, "G");
        }
        let w = w/1024;
        if w < 1024 {
            w.fmt(f)?;
            return write!(f, "T");
        }  
        let w = w/1024;
        w.fmt(f)?;
        write!(f, "P")
    }
}



#[test]
fn test_format_number() {
    assert_eq!("1023", format!("{}", HrSize(1023)));
    assert_eq!("1K", format!("{}", HrSize(1024)));
    assert_eq!("1K", format!("{}", HrSize(1025)));
    assert_eq!("1025", format!("{:#}", HrSize(1025)));

    assert_eq!("1023K", format!("{}", HrSize(1024*1024-1)));
    assert_eq!("1M", format!("{}", HrSize(1024*1024)));

    assert_eq!("1023M", format!("{}", HrSize(1024*1024*1024-1)));
    assert_eq!("1G", format!("{}", HrSize(1024*1024*1024)));

    assert_eq!("1023G", format!("{}", HrSize(1024*1024*1024*1024-1)));
    assert_eq!("1T", format!("{}", HrSize(1024*1024*1024*1024)));

    assert_eq!("1023T", format!("{}", HrSize(1024*1024*1024*1024*1024-1)));
    assert_eq!("1P", format!("{}", HrSize(1024*1024*1024*1024*1024)));
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


struct Statistics {
    count: u64,
    size_max: u64,
    size_min: u64,
    size_total: u64, 
    sizes: Vec<u64>,
    do_median: bool,
}

impl Default for Statistics {
    fn default() -> Self {
        Self { 
            count: 0, 
            size_max: 0, 
            size_min: u64::MAX, 
            size_total: 0, 
            sizes: vec![],
            do_median: false,
        }
    }
}

impl Statistics {
    fn apply(&mut self, size: u64) {
        self.count += 1;
        self.size_total += size;
        self.size_max = self.size_max.max(size);
        self.size_min = self.size_min.min(size);
        if self.do_median { self.sizes.push(size); }     
    }

    fn result(&mut self) -> Option<(u64, HrSize, HrSize, HrSize, HrSize, HrSize)> {
        if self.count == 0 { return None; }
        let size_avg = self.size_total/self.count;
        
        let size_median = if self.do_median {
            self.sizes.sort();
            let lh = self.sizes.len()/2;
            if self.sizes.len().is_multiple_of(2) {
                (self.sizes[lh] + self.sizes[lh-1])/2
            } else {
                self.sizes[lh]
            }
        } else {
            0
        };

        let size_total = HrSize(self.size_total);
        let size_min = HrSize(self.size_min);
        let size_max = HrSize(self.size_max);
        let size_avg = HrSize(size_avg);
        let size_median = HrSize(size_median);
        Some((self.count, size_total, size_min, size_max, size_avg, size_median))         
    }
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
            if let Some(version_id) = get.version_id {
                bucket.get_object_version(source_path, &version_id, &mut output_file)?;
            } else {
                bucket.get_object(source_path, &mut output_file)?;
            }
            
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
        ObsCommand::PutStr(put) => {
            let (bucket_name, target_path) = split_bucket(&put.remote);
            let bucket = client.bucket(bucket_name.to_owned())?;
            let source = put.string;
            bucket.put_object(target_path, source)?;
            Ok(JsonValue::Bool(true))
        }
        ObsCommand::Ls(ls) => {
            let (bucket_name, bucket_path) = split_bucket(&ls.remote);
            let bucket = client.bucket(bucket_name.to_owned())?;
            if ls.long {
                println!("etag\tstorage_class\towner\ttype\tsize\tlast_modified\tkey");
                println!("----\t-------------\t-----\t----\t----\t-------------\t---");
            } else {
                println!("type\tsize\tlast_modified\tkey");
                println!("----\t----\t-------------\t---");
            }

            let mut marker = ls.marker;
            let pages = if let Some(pages) = ls.pages { 
                pages 
            } else if ls.all {
                usize::MAX
            } else {
                1
            };

            let mut s = Statistics { do_median: true, ..Default::default() };

            for _ in 0..pages {
                let list_request = ListObjectsRequest {
                    prefix: Some(bucket_path),
                    marker: marker.as_deref(),
                    max_keys: ls.max_keys,
                    ..Default::default()
                };
                let list = bucket.list_objects(list_request)?;
                let Some(contents) = list.contents else { break };

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
                    let is_std_entry = !key.ends_with('/');

                    if ls.raw || is_std_entry {
                        let type_ = type_.as_deref().unwrap_or("-");
                        let owner = owner.as_ref().map(|s| &s.id as &str).unwrap_or("-");
                        if ls.long {
                            println!("{etag}\t{storage_class}\t{owner}\t{type_}\t{size}\t{last_modified}\t{key}")
                        } else {
                            println!("{type_}\t{size}\t{last_modified}\t{key}")
                        }
                    }

                    if is_std_entry {
                        s.apply(size);
                    }
                }

                marker = list.next_marker;

                if marker.is_none() { break }
            }
            if ls.long {
                println!("----\t-------------\t-----\t----\t----\t-------------\t---");
                println!("etag\tstorage_class\towner\ttype\tsize\tlast_modified\tkey");
            } else {
                println!("----\t----\t-------------\t---");
                println!("type\tsize\tlast_modified\tkey");
            }
            if let Some(marker) = marker {
                println!("\nnext_marker: {marker}");
            }

            if let Some((count, size_total, size_min, size_max, size_avg, size_median)) = s.result() {
                println!("\nstats: count={count} size_total={size_total} size_min={size_min} size_max={size_max} size_avg={size_avg} size_median={size_median}");
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
            if ls.long {
                println!("version_id\tlatest\tetag\tstorage_class\towner\ttype\tsize\tlast_modified\tkey")
            } else {
                println!("version_id\tlatest\ttype\tsize\tlast_modified\tkey")
            }
            println!("---------------------------------------");
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
                let version_id = version_id.as_deref().unwrap_or("-");
                let latest = match is_latest {
                    Some(true) => "latest",
                    Some(false) => "-",
                    None => "?"
                };
                if ls.long {
                    println!("{version_id}\t{latest}\t{etag}\t{storage_class}\t{owner}\t{type_}\t{size}\t{last_modified}\t{key}")
                } else {
                    println!("{version_id}\t{latest}\t{type_}\t{size}\t{last_modified}\t{key}")
                }
            }
            if let Some(next_key_marker) = list.next_key_marker {
                println!("next_key_marker: {next_key_marker}")
            }
            Ok(JsonValue::Bool(true))
        }
        ObsCommand::Du(du) => {
            let (bucket_name, bucket_path) = split_bucket(&du.remote);
            let bucket = client.bucket(bucket_name.to_owned())?;
 
            let mut marker = du.marker;
            let pages = if let Some(pages) = du.pages { 
                pages 
            } else {
                usize::MAX
            };

            let mut s: BTreeMap<Vec<String>, Statistics> = BTreeMap::new(); 

            for _ in 0..pages {
                let list_request = ListObjectsRequest {
                    prefix: Some(bucket_path),
                    marker: marker.as_deref(),
                    max_keys: du.max_keys,
                    ..Default::default()
                };
                let list = bucket.list_objects(list_request)?;
                let Some(contents) = list.contents else { break };

                for ListObjectsContents { 
                    key, 
                    size, 
                    ..
                } in contents {
                    let is_std_entry = !key.ends_with('/');

                    if is_std_entry {
                        let Some(eff_key) = key.strip_prefix(bucket_path) else {
                            eprintln!("Got path not matching the prefix (`{bucket_path}`): `{key}`");
                            continue;
                        };

                        let eff_key = eff_key.strip_prefix('/').unwrap_or(eff_key);

                        let eff_key: Vec<String> = eff_key.split('/').take(du.depth).map(str::to_owned).collect();
                        for n in 0..=eff_key.len() {
                            let k = eff_key[0..n].to_vec();
                            s.entry(k).or_default().apply(size);
                        }
                    }
                }
                print!(".");
                _ = std::io::stdout().flush();

                marker = list.next_marker;

                if marker.is_none() { break }
            }
            println!();
            println!("   count  size_total   size_min   size_max   size_avg dir");
            println!("-------- ----------- ---------- ---------- ---------- ---");

            for (k, mut v) in s {
                if let Some((count, size_total, size_min, size_max, size_avg, _)) = v.result() {
                    let key = k.join("/");
                    println!("{count:>8} {size_total:>10} {size_min:>9} {size_max:>9} {size_avg:>9} {key}")
                }
            }
            println!("-------- ----------- ---------- ---------- ---------- ---");
            println!("   count  size_total   size_min   size_max   size_avg dir");

            if let Some(marker) = marker {
                println!("\nnext_marker: {marker}");
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