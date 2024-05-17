use http::{HeaderMap, Method};
use url::Url;

use crate::{model::obs::{BucketMeta, ObjectMeta}, CloudRuError, Result};

use super::urltools::WithVar;

pub enum FsType {
    Obs,
    Pfs
}


pub struct WriteOp {
    pub method: Method,
    pub verb_and_position: Option<(&'static str, u64)>
}
impl WriteOp {
    pub fn modify_url(&self, url: Url) -> Url {
        if let Some((verb, pos)) = self.verb_and_position {
            url
                .with_var_key(verb)
                .with_var("position", &format!("{}", pos))
        } else {
            url
        }
    }
}



impl FsType {
    pub fn from_bucket_meta(bucket_meta: &BucketMeta) -> Self {
        match bucket_meta.fs_file_interface.as_deref() {
            Some("Enabled") => FsType::Pfs,
            _ => FsType::Obs,
        }
    }

    /// Returns (if possible) the correct http method + write-op verb ("append" or "modify") depending on the FS
    /// 
    /// Also checks `pos`(position to start writing) and `len` (actual length of the file) for consistency with the op.
    pub fn eval_write_op(&self, pos: u64, len: u64) -> Result<WriteOp> {
        /*
POST /ObjectName?append&position=Position HTTP/1.1 
Host: bucketname.obs.region.example.com
Content-Type: application/xml 
Content-Length: length
Authorization: authorization
Date: date
<Optional Additional Header> 
<object Content>
        */
        /*
PUT /ObjectName?modify&position=Position HTTP/1.1
Host: bucketname.obs.region.example.com
Content-Type: type
Content-Length: length
Authorization: authorization
Date: date
<object Content>
         */
        match self {
            Self::Obs => {
                let verb = "append";
                if pos != len { return Err(CloudRuError::InconsistentFsOp { verb, pos, len }) }
                Ok(WriteOp { method: Method::POST, verb_and_position: Some((verb, pos)) })
            }
            Self::Pfs => if pos == 0 && len == 0 { //the object is empty or does not exist yet
                Ok(WriteOp { method: Method::PUT, verb_and_position: None })
            } else {
                let verb = "modify";
                if pos > len { return Err(CloudRuError::InconsistentFsOp { verb, pos, len }) }
                Ok(WriteOp { method: Method::PUT, verb_and_position: Some((verb, pos)) })
            }
        }
    }
}

impl From<&BucketMeta> for FsType {
    fn from(bucket_meta: &BucketMeta) -> Self {
        Self::from_bucket_meta(bucket_meta)
    }
}

pub fn extract_bucket_meta(headers: &HeaderMap) -> BucketMeta {
    let get_str = |key: &str| headers.get(key)
        .and_then(|w| w.to_str().ok())
        .map(|w| w.to_owned());

    let bucket_location = get_str("x-obs-bucket-location");
    let storage_class = get_str("x-obs-storage-class");
    let version = get_str("x-obs-version");
    let fs_file_interface = get_str("x-obs-fs-file-interface");
    let epid = get_str("x-obs-epid");
    let az_redundancy = get_str("x-obs-az-redundancy");

    BucketMeta { bucket_location, storage_class, version, fs_file_interface, epid, az_redundancy }
}

pub fn extract_object_meta(headers: &HeaderMap) -> ObjectMeta {

    let get_int = |key: &str| headers.get(key)
        .and_then(|w| w.to_str().ok())
        .and_then(|w| w.parse().ok());

    let get_str = |key: &str| headers.get(key)
        .and_then(|w| w.to_str().ok())
        .map(|w| w.to_owned());

    let content_length = get_int("content-length");
    let content_type = get_str("content-type");
    //Last-Modified: WED, 01 Jul 2015 01:19:21 GMT
    let last_modified = get_str("last-modified");

    let expiration = get_str("x-obs-expiration");
    let website_redirect_location = get_str("x-obs-website-redirect-location");
    let version_id = get_str("x-obs-version-id");
    let object_type = get_str("x-obs-object-type");
    let next_append_position = get_int("x-obs-next-append-position");
    let storage_class = get_str("x-obs-storage-class");

    ObjectMeta { content_length, content_type, last_modified, expiration, 
        website_redirect_location, version_id, object_type, next_append_position, storage_class 
    }
}
