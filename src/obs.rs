use std::{io::{self, Read, Seek, SeekFrom, Write}, sync::Arc};
use crate::*;
use reqwest::{Url, blocking::{Request, Client as HttpClient, Body, RequestBuilder}, Method, header::HeaderValue};
use error::ParameterKind;
use mauth_obs::*;
use serde_derive::Deserialize;
use tracing::{debug, instrument, Level, enabled};
use CloudRuInnerError;
use CloudRuError;


pub struct ObsClient {
    endpoint: String,
    aksk: AkSk,
    http_client: Arc<HttpClient>,
}

impl ObsClient {
    pub fn new(endpoint: String, aksk: AkSk, http_client: Arc<HttpClient>) -> Self { Self { endpoint, http_client, aksk } }
    pub fn bucket(&self, bucket_name: String) -> Result<Bucket> { 
        Bucket::new(
            bucket_name, 
            self.endpoint.clone(), 
            self.aksk.clone(), 
            self.http_client.clone()
        ) }
}

#[derive(Debug, Clone)]
pub struct Bucket {
    bucket_name: String,
    bucket_url: Url,
    host: HeaderValue,
    aksk: AkSk,
    http_client: Arc<HttpClient>,
}

trait Signer {
    fn timestamp_and_sign(self, bucket_name: &str, aksk: &AkSk) -> Result<Request>;
}

impl Signer for RequestBuilder {
    fn timestamp_and_sign(self, bucket_name: &str, aksk: &AkSk) -> Result<Request> {
        let mut request = self.build()?;
        let dt = time::OffsetDateTime::now_utc();
        time_stamp_and_sign(bucket_name, &mut request, dt, &aksk.ak, &aksk.sk)?;
        Ok(request)
    }
}

trait WithVar {
    fn with_var<T: AsRef<str>>(self, var: &str, val: T) -> Url;
    fn with_var_opt<T: AsRef<str>>(self, var: &str, val: Option<T>) -> Url;
    fn with_var_key(self, var: &str) -> Url;
}

impl WithVar for Url {
    fn with_var_opt<T: AsRef<str>>(mut self, var: &str, val: Option<T>) -> Url {
        if let Some(val) = val {
            self.query_pairs_mut().append_pair(var, val.as_ref());
        }
        self
    }

    fn with_var<T: AsRef<str>>(mut self, var: &str, val: T) -> Url {
        self.query_pairs_mut().append_pair(var, val.as_ref());
        self
    }

    fn with_var_key(mut self, var: &str) -> Url {
        self.query_pairs_mut().append_key_only(var);
        self
    }

}

#[derive(Debug, Default)]
pub struct ListBucketRequest<'t> {
    pub prefix: Option<&'t str>,
    pub marker: Option<&'t str>,
    pub max_keys: Option<u32>,
    pub delimiter: Option<&'t str>,
    pub key_marker: Option<&'t str>,
    pub version_id_marker: Option<&'t str>,
}

/*
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>
<ListBucketResult xmlns=\"http://obs.hc.sbercloud.ru/doc/2015-06-30/\">
    <Name>rust-api-test</Name>
    <Prefix></Prefix>
    <KeyCount>2</KeyCount>
    <MaxKeys>1000</MaxKeys>
    <IsTruncated>false</IsTruncated>
    <Contents>
        <Key>test.txt</Key>
        <LastModified>2023-08-20T11:04:49.119Z</LastModified>
        <ETag>\"bf0b7283b196369ba8723a79750d937b\"</ETag>
        <Size>35</Size>
        <StorageClass>STANDARD</StorageClass>
    </Contents>
    <Contents>
        <Key>test2.txt</Key>
        <LastModified>2023-08-20T11:04:49.250Z</LastModified>
        <ETag>\"bf0b7283b196369ba8723a79750d937b\"</ETag>
        <Size>35</Size>
        <StorageClass>STANDARD</StorageClass>
    </Contents>
</ListBucketResult>
*/
#[derive(Deserialize, Debug)]
pub struct ListBucketResult {
    #[serde(rename="Name")]
    pub name: String,
    
    #[serde(rename="Prefix")]
    pub prefix: String,
    
    #[serde(rename="KeyCount")]
    pub key_count: Option<u64>,
    
    #[serde(rename="MaxKeys")]
    pub max_keys: Option<u64>,

    #[serde(rename="IsTruncated")]
    pub is_truncated: Option<bool>,

    #[serde(rename="Delimiter")]
    pub delimeter: Option<String>,

    #[serde(rename="Marker")]
    pub marker: Option<String>,

    #[serde(rename="NextMarker")]
    pub next_marker: Option<String>,

    //#[serde(rename="CommonPrefixes")]
    //pub common_prefixes: Option<String>,

    #[serde(rename="Contents")]
    pub contents: Vec<ListBucketContents>,
}

#[derive(Deserialize, Debug)]
pub struct ListBucketContents {
    #[serde(rename="Key")]
    pub key: String,

    #[serde(rename="LastModified")]
    pub last_modified: String,
    
    #[serde(rename="ETag")]
    pub etag: String,
    
    #[serde(rename="Type")]
    pub type_: Option<String>,
    
    #[serde(rename="Size")]
    pub size: u64,
    
    #[serde(rename="StorageClass")]
    pub storage_class: String,

    #[serde(rename="Owner")]
    pub owner: Option<Owner>,
}

#[derive(Deserialize, Debug)]
pub struct Owner {
    #[serde(rename="ID")]
    pub id: String,

    #[serde(rename="DisplayName")]
    pub display_name: Option<String>,
}


macro_rules! bail_on_failure {
    ($result:expr) => {
        if !$result.status().is_success() {
            let status = $result.status();
            let err = $result.text().cx("error text in bail_on_failure")?;
            return Err(CloudRuInnerError::API(status, err).into());
        }        
    };
}


impl Bucket {
    pub fn new(bucket_name: String, obs_endpoint: String, aksk: AkSk, http_client: Arc<HttpClient>) -> Result<Self> {
        let mut bucket_url: Url = obs_endpoint.parse()?;
        let bucket_host = bucket_url.host()
            .ok_or(CloudRuInnerError::Parameter(ParameterKind::S3BucketUrl))?;
        let host = format!("{}.{}", bucket_name, bucket_host);
        bucket_url.set_host(Some(&host))?;
        let host = host.parse()?;
        Ok(Self { bucket_name, bucket_url, host, aksk, http_client })
    }

    #[inline]
    fn url(&self, path: impl AsRef<str>) -> Url {
        let mut url = self.bucket_url.clone();
        url.set_path(path.as_ref());
        url
    }

    #[inline]
    pub fn make_url(&self, path: impl AsRef<str>) -> String {
        self.url(path).to_string()
    }

    #[instrument]
    pub fn list_objects(&self, request: ListBucketRequest<'_>) -> Result<ListBucketResult> {
        
        let url = self.url("/")
            .with_var_opt("prefix", request.prefix)
            .with_var_opt("marker", request.marker)
            .with_var_opt("max_keys", request.max_keys.map(|s| format!("{s}")))
            .with_var_opt("delimiter", request.delimiter)
            .with_var_opt("key_marker", request.key_marker)
            .with_var_opt("version_id_marker", request.version_id_marker)
            ;

        let request = self.http_client.request(Method::GET, url)
            .header("host", self.host.clone())
            .timestamp_and_sign(&self.bucket_name, &self.aksk)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request)?;
        bail_on_failure!(result);

        let p: ListBucketResult = if enabled!(Level::DEBUG) {
            let text = result.text()?;
            debug!(response_text=?&text);
            serde_xml_rs::from_str(&text)?
        } else {
            serde_xml_rs::from_reader(result)?
        };
        Ok(p)

    }

    pub fn list(&self, prefix: Option<&str>) -> Result<ListBucketResult> {
        self.list_objects(ListBucketRequest { prefix, ..Default::default() })
    }

    fn start_request(&self, request: RequestBuilder) -> RequestBuilder {
        request.header("host", self.host.clone())
    }

    fn sign_request(&self, request: RequestBuilder) -> Result<Request> {
        request.timestamp_and_sign(&self.bucket_name, &self.aksk)
    }

    /// get object at `remote_path` and write its data to `w`
    pub fn get_object<W: Write>(&self, remote_path: impl AsRef<str>, w: &mut W) -> Result<()> {
        let request = self.http_client.request(Method::GET, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let mut result = self.http_client.execute(request)?;
        bail_on_failure!(result);
        
        result.copy_to(w)?;
        Ok(())
    }

    /// put object at `remote_path` filling it with data read from `input`
    pub fn put_object<I>(&self, remote_path: impl AsRef<str>, input: I) -> Result<()> where Body: From<I> {
        let request = self.http_client.request(Method::PUT, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = request.body(input);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request)?;
        bail_on_failure!(result);

        Ok(())
    }

    /// delete object at `remote_path`
    pub fn delete_object(&self, remote_path: impl AsRef<str>) -> Result<()> {
        let request = self.http_client.request(Method::DELETE, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request)?;
        bail_on_failure!(result);

        Ok(())
    }

    /// copy object from `source_bucket`:`source_path` to `remote_path`
    pub fn copy_object(&self, remote_path: impl AsRef<str>, source_bucket: impl AsRef<str>, source_path: impl AsRef<str>,) -> Result<()> {
        let request = self.http_client.request(Method::PUT, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = request.header(
            "x-obs-copy-source", 
            format!("/{}/{}", source_bucket.as_ref(), source_path.as_ref())
        );
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request)?;
        bail_on_failure!(result);

        Ok(())
    }

    /// get object's metadata
    /// @see https://support.hc.sbercloud.ru/api/obs/obs_04_0084.html
    pub fn get_object_meta(&self, remote_path: impl AsRef<str>) -> Result<ObjectMeta> {
        let request = self.http_client.request(Method::HEAD, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request)?;
        bail_on_failure!(result);

        let content_length = result.headers().get("content-length")
            .and_then(|w| w.to_str().ok())
            .and_then(|w| w.parse().ok());
        let content_type = result.headers().get("content-type")
            .and_then(|w| w.to_str().ok())
            .map(|w| w.to_owned());
        //Last-Modified: WED, 01 Jul 2015 01:19:21 GMT
        let last_modified = result.headers().get("last-modified")
            .and_then(|w| w.to_str().ok())
            .map(|w| w.to_owned());
        Ok(ObjectMeta { content_length, content_type, last_modified })                
    }

    /// get object reader. The reader implements [std::io::Read].
    pub fn object_reader(&self, remote_path: impl AsRef<str>) -> Result<ObjectReader> {
        let remote_path = remote_path.as_ref().to_string();
        let metadata = self.get_object_meta(&remote_path)?;
        let len = metadata.content_length.ok_or_else(|| CloudRuError::new(
            CloudRuInnerError::UnknownObjectLength, 
            remote_path.clone()
        ))?;

        let bucket = self.clone();
        let client = self.http_client.clone();
        
        let pos = 0;
         
        Ok(ObjectReader { remote_path, bucket, client, metadata, pos, len })
    }

    /// get object writer. The writer implements [std::io::Write]. 
    /// Note that currently the object must not exist or must have zero length
    pub fn object_writer(&self, remote_path: impl AsRef<str>) -> Result<ObjectWriter> {
        let remote_path = remote_path.as_ref().to_string();
        let bucket = self.clone();
        let client = self.http_client.clone();
        let pos = 0;
         
        Ok(ObjectWriter { remote_path, bucket, client, pos })
    }

}


/// Object metadata returned by [obs::Bucket::get_object_meta]
pub struct ObjectMeta {
    /// Length of the object in bytes as seen by OBS 
    pub content_length: Option<u64>,
    /// Type of the object's content
    pub content_type: Option<String>,
    /// Date the object was last modified, like "WED, 01 Jul 2015 01:19:21 GMT"
    pub last_modified: Option<String>,
}

pub struct ObjectReader {
    remote_path: String,
    bucket: Bucket,
    client: Arc<HttpClient>,
    metadata: ObjectMeta,
    len: u64,
    pos: u64,
}

impl ObjectReader {
    /// updates a cached copy of the object's metadata and returns reference to it
    pub fn get_meta(&mut self) -> Result<&ObjectMeta> {
        self.metadata = self.bucket.get_object_meta(&self.remote_path)?;
        Ok(&self.metadata)
    }
    /// returns a cached copy of the object's metadata
    pub fn meta(&self) -> &ObjectMeta { &self.metadata }
    pub fn len(&self) -> u64 { self.len }
    pub fn pos(&self) -> u64 { self.pos }
}

impl Read for ObjectReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0)
        }

        let (first, last) = (self.pos, self.pos + buf.len() as u64 - 1);
        let range = format!("bytes={first}-{last}");

        let request = self.client.request(Method::GET, self.bucket.url(&self.remote_path));
        let request: RequestBuilder = self.bucket.start_request(request);
        let request = request.header("range", range);
        let request = self.bucket.sign_request(request)?;
    
        debug!(request_full=?request);

        let mut result = self.client.execute(request).cx("Client::execute")?;
        if result.status().is_success() {
            if result.status().as_u16() != 206 { //Partial content
                return Err(io::Error::new(io::ErrorKind::Unsupported, "returning ranges not suppoerted"))
            }
            let mut w = buf;
            let count = result.copy_to(&mut w).cx("copy_to")?;
            self.pos += count;
            Ok(count as usize)
        } else {
            Err(CloudRuInnerError::API(result.status(), result.text().cx("text")?).into())
        }
    }
}

impl Seek for ObjectReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            SeekFrom::Start(pos) =>
                pos,
            SeekFrom::Current(off) => 
                self.pos.checked_add_signed(off).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
            SeekFrom::End(off) => 
                self.len.checked_add_signed(off).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
        };
        Ok(self.pos)
    }
}



pub struct ObjectWriter {
    remote_path: String,
    bucket: Bucket,
    client: Arc<HttpClient>,
    pos: u64,
}

impl ObjectWriter {
    /// Synchronizes cached position with the length of the actual object, so that we can resume appending to it.
    /// The object must exist and must be created in append mode.
    pub fn sync_position(&mut self) -> Result<u64> {
        let meta = self.bucket.get_object_meta(&self.remote_path)?;
        self.pos = meta.content_length.ok_or_else(
            || CloudRuError::new(CloudRuInnerError::UnknownObjectLength, self.remote_path.to_owned())
        )?;
        Ok(self.pos)
    }
}

impl Write for ObjectWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
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

        let url = self.bucket.url(&self.remote_path)
            .with_var_key("append")
            .with_var("position", &format!("{}", self.pos));

        let request = self.client.request(Method::POST, url);
        let request: RequestBuilder = self.bucket.start_request(request);
        let request = request.body(Vec::from(buf));
        let request = self.bucket.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.client.execute(request).cx("Client::execute")?;
        bail_on_failure!(result);
        let count = buf.len();
        self.pos += count as u64;
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

