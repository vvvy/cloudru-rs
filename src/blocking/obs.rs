use std::{io::{self, Read, Seek, SeekFrom, Write}, sync::Arc};
use reqwest::{blocking::{Body, Request, RequestBuilder}, header::{HeaderMap, HeaderValue}, Method, Url};
use tracing::{debug, instrument, Level, enabled};

use crate::shared::{mauth_obs::*, obs::{extract_bucket_meta, extract_object_meta}};
use error::ParameterKind;
use CloudRuError;

pub use crate::model::obs::*;
use crate::shared::urltools::*;
use self::shared::obs::FsType;

use super::*;
use crate::*;


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

struct R<'r> { r: &'r mut Request }
impl<'r> RequestW for R<'r> {
    fn method(&self) -> &Method { self.r.method() }
    fn headers(&self) -> &HeaderMap { self.r.headers() }
    fn headers_mut(&mut self) -> &mut HeaderMap { self.r.headers_mut() }
    fn url(&self) -> &Url { self.r.url() }
}


trait Signer {
    fn timestamp_and_sign(self, bucket_name: &str, aksk: &AkSk) -> Result<Request>;
}

impl Signer for RequestBuilder {
    fn timestamp_and_sign(self, bucket_name: &str, aksk: &AkSk) -> Result<Request> {
        let mut request = self.build()?;
        let dt = time::OffsetDateTime::now_utc();
        time_stamp_and_sign(bucket_name, &mut R { r: &mut request }, dt, &aksk.ak, &aksk.sk)?;
        Ok(request)
    }
}

macro_rules! bail_on_failure {
    ($result:expr) => {
        if !$result.status().is_success() {
            let status = $result.status();
            let err = $result.text().cx("error text in bail_on_failure")?;
            return Err(CloudRuError::API(status, err).into());
        }        
    };
}


impl Bucket {
    pub fn new(bucket_name: String, obs_endpoint: String, aksk: AkSk, http_client: Arc<HttpClient>) -> Result<Self> {
        let mut bucket_url: Url = obs_endpoint.parse()?;
        let bucket_host = bucket_url.host()
            .ok_or(CloudRuError::Parameter(ParameterKind::S3BucketUrl))?;
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

        Ok(extract_object_meta(result.headers()))                 
    }

    /// get bucket metadata
    pub fn get_bucket_meta(&self) -> Result<BucketMeta> {
        let request = self.http_client.head(self.url("/"));
        let request: RequestBuilder = self.start_request(request);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request)?;
        bail_on_failure!(result);

        Ok(extract_bucket_meta(result.headers()))
    }

    /// get object reader. The reader implements [std::io::Read].
    pub fn object_reader(&self, remote_path: impl AsRef<str>) -> Result<ObjectReader> {
        let remote_path = remote_path.as_ref().to_string();
        let metadata = self.get_object_meta(&remote_path)?;
        let len = metadata.content_length.ok_or_else(||
            CloudRuError::UnknownObjectLength(remote_path.clone())
        )?;

        let bucket = self.clone();
        let client = self.http_client.clone();
        
        let pos = 0;
         
        Ok(ObjectReader { remote_path, bucket, client, metadata, pos, len })
    }

    /// get object writer. The writer implements [std::io::Write]. 
    /// Note that currently
    /// a) for obs, the object must not exist or must have zero length, otherwise the error is returned
    /// b) for pfs, the object gets overwritten if it exists
    pub fn object_writer(&self, remote_path: impl AsRef<str>) -> Result<ObjectWriter> {
        let remote_path = remote_path.as_ref().to_string();
        let bucket = self.clone();
        let pos = 0;

        //check if we are pfs
        let bucket_meta = bucket.get_bucket_meta()?;
        let fs_type = FsType::from_bucket_meta(&bucket_meta);
         
        Ok(ObjectWriter { remote_path, bucket, fs_type, pos })
    }

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
            Err(CloudRuError::API(result.status(), result.text().cx("text")?).into())
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
    fs_type: FsType,
    pos: u64,
}

impl ObjectWriter {
    /// Synchronizes cached position with the length of the actual object, so that we can resume appending to it.
    /// The object must exist and must be created in append mode.
    pub fn sync_position(&mut self) -> Result<u64> {
        let meta = self.bucket.get_object_meta(&self.remote_path)?;
        self.pos = meta.content_length.ok_or_else(
            || CloudRuError::UnknownObjectLength(self.remote_path.clone())
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
        let write_op = self.fs_type.eval_write_op(self.pos, self.pos)?;
        let url = write_op.modify_url(self.bucket.url(&self.remote_path));

        let request = self.bucket.http_client.request(write_op.method, url)
            .header("Content-Length", format!("{}", buf.len()));
        let request: RequestBuilder = self.bucket.start_request(request);
        let request = request.body(Vec::from(buf));
        let request = self.bucket.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.bucket.http_client.execute(request).cx("Client::execute")?;
        bail_on_failure!(result);
        let count = buf.len();
        self.pos += count as u64;
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

