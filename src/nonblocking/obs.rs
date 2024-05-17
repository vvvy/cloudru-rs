use std::io;

use bytes::Bytes;

use tracing::{debug, instrument};
use reqwest::{header::{HeaderMap, HeaderValue}, Body, Method, Request, RequestBuilder};
use url::Url;

pub use crate::model::obs::*;
use self::{error::ParameterKind, shared::obs::FsType};
use crate::shared::{mauth_obs::*, obs::{extract_bucket_meta, extract_object_meta}, urltools::WithVar};
use super::*;
use crate::*;


pub struct ObsClient {
    endpoint: String,
    credentials: Credentials,
    http_client: HttpClient,
}

impl ObsClient {
    pub fn new(endpoint: String, credentials: Credentials, http_client: HttpClient) -> Self { Self { endpoint, http_client, credentials } }
    pub fn bucket(&self, bucket_name: String) -> Result<Bucket> { 
        Bucket::new(
            bucket_name, 
            self.endpoint.clone(), 
            self.credentials.clone(), 
            self.http_client.clone()
        ) }
}

#[derive(Debug, Clone)]
pub struct Bucket {
    bucket_name: String,
    bucket_url: Url,
    host: HeaderValue,
    credentials: Credentials,
    http_client: HttpClient,
}

struct R<'r> { r: &'r mut Request }
impl<'r> RequestW for R<'r> {
    fn method(&self) -> &Method { self.r.method() }
    fn headers(&self) -> &HeaderMap { self.r.headers() }
    fn headers_mut(&mut self) -> &mut HeaderMap { self.r.headers_mut() }
    fn url(&self) -> &Url { self.r.url() }
}


trait Signer {
    fn timestamp_and_sign(self, bucket_name: &str, credentials: &Credentials) -> Result<Request>;
}

impl Signer for RequestBuilder {
    fn timestamp_and_sign(self, bucket_name: &str, credentials: &Credentials) -> Result<Request> {
        let mut request = self.build()?;
        let dt = time::OffsetDateTime::now_utc();
        time_stamp_and_sign(bucket_name, &mut R { r: &mut request }, dt, &credentials.ak, &credentials.sk)?;
        Ok(request)
    }
}

macro_rules! bail_on_failure {
    ($result:expr) => {
        if !$result.status().is_success() {
            let status = $result.status();
            let err = $result.text().await.cx("error text in bail_on_failure")?;
            return Err(CloudRuError::API(status, err));
        }        
    };
}


impl Bucket {
    pub fn new(bucket_name: String, obs_endpoint: String, credentials: Credentials, http_client: HttpClient) -> Result<Self> {
        let mut bucket_url: Url = obs_endpoint.parse()?;
        let bucket_host = bucket_url.host()
            .ok_or(CloudRuError::Parameter(ParameterKind::S3BucketUrl))?;
        let host = format!("{}.{}", bucket_name, bucket_host);
        bucket_url.set_host(Some(&host))?;
        let host = host.parse()?;
        Ok(Self { bucket_name, bucket_url, host, credentials, http_client })
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
    pub async fn list_objects(&self, request: ListObjectsRequest<'_>) -> Result<ListObjectsResult> {
        
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
            .timestamp_and_sign(&self.bucket_name, &self.credentials)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request).await?;
        bail_on_failure!(result);

        let text = result.text().await?;
        debug!(response_text=?&text);
        let p = serde_xml_rs::from_str(&text)?;

        Ok(p)
    }

    pub async fn list(&self, prefix: Option<&str>) -> Result<ListObjectsResult> {
        self.list_objects(ListObjectsRequest { prefix, ..Default::default() }).await
    }

    fn start_request(&self, request: RequestBuilder) -> RequestBuilder {
        request.header("host", self.host.clone())
    }

    fn sign_request(&self, request: RequestBuilder) -> Result<Request> {
        request.timestamp_and_sign(&self.bucket_name, &self.credentials)
    }


    /// get object at `remote_path`
    pub async fn get_object(&self, remote_path: impl AsRef<str>) -> Result<Bytes> {
        let request = self.http_client.request(Method::GET, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request).await?;
        bail_on_failure!(result);
        let rv = result.bytes().await?;
        
        Ok(rv)
    }


    /// put object at `remote_path` filling it with data read from `input`
    pub async fn put_object<I>(&self, remote_path: impl AsRef<str>, input: I) -> Result<()> where Body: From<I> {
        let request = self.http_client.request(Method::PUT, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = request.body(input);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request).await?;
        bail_on_failure!(result);

        Ok(())
    }

    /// delete object at `remote_path`
    pub async fn delete_object(&self, remote_path: impl AsRef<str>) -> Result<()> {
        let request = self.http_client.request(Method::DELETE, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request).await?;
        bail_on_failure!(result);

        Ok(())
    }

    /// copy object from `source_bucket`:`source_path` to `remote_path`
    pub async fn copy_object(&self, remote_path: impl AsRef<str>, source_bucket: impl AsRef<str>, source_path: impl AsRef<str>,) -> Result<()> {
        let request = self.http_client.request(Method::PUT, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = request.header(
            "x-obs-copy-source", 
            format!("/{}/{}", source_bucket.as_ref(), source_path.as_ref())
        );
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request).await?;
        bail_on_failure!(result);

        Ok(())
    }

    /// get object's metadata
    /// @see https://support.hc.sbercloud.ru/api/obs/obs_04_0084.html
    pub async fn get_object_meta(&self, remote_path: impl AsRef<str>) -> Result<ObjectMeta> {
        let request = self.http_client.request(Method::HEAD, self.url(remote_path));
        let request: RequestBuilder = self.start_request(request);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request).await?;
        bail_on_failure!(result);

        Ok(extract_object_meta(result.headers()))             
    }

    /// get bucket metadata
    pub async fn get_bucket_meta(&self) -> Result<BucketMeta> {
        let request = self.http_client.head(self.url("/"));
        let request: RequestBuilder = self.start_request(request);
        let request = self.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.http_client.execute(request).await?;
        bail_on_failure!(result);

        Ok(extract_bucket_meta(result.headers()))
    }

    /// create file-like IO object that track r/w position
    pub async fn object_io(&self, remote_path: impl AsRef<str>) -> Result<ObjectIO> {
        let remote_path = remote_path.as_ref().to_string();

        // yield 0 if it is 404 error
        let len = match self.get_object_meta(&remote_path).await {
            Ok(metadata) => metadata.content_length.ok_or_else(
                || CloudRuError::UnknownObjectLength(remote_path.clone())
            )?,
            Err(e) => match e {
                CloudRuError::API(n, _) if n == 404 => Ok(0),
                _ => Err(e),
            }?
        };

        let bucket = self.clone();
        let pos = 0;

        //check if we are pfs
        let bucket_meta = bucket.get_bucket_meta().await?;
        let fs_type = FsType::from_bucket_meta(&bucket_meta);
         
        Ok(ObjectIO { remote_path, bucket, fs_type, pos, len })
    }


}


pub struct ObjectIO {
    remote_path: String,
    bucket: Bucket,
    fs_type: FsType,
    pos: u64,
    len: u64,
} 

impl ObjectIO {
    /// Synchronizes cached position with the length of the actual object, so that we can resume appending to it.
    /// The object must exist and must be created in append mode.
    pub async fn sync_position(&mut self) -> Result<u64> {
        let meta = self.bucket.get_object_meta(&self.remote_path).await?;
        self.len = meta.content_length.ok_or_else(
            || CloudRuError::UnknownObjectLength(self.remote_path.clone())
        )?;
        if self.pos > self.len { self.pos = self.len }
        Ok(self.len)
    }

    /// Read/write position
    pub fn pos(&self) -> u64 { self.pos }
    /// Current length of the entire object
    pub fn len(&self) -> u64 { self.len }

    /// read `len` bytes from the bucket 
    pub async fn read(&mut self, len: usize) -> Result<Bytes> {
        if len == 0 {
            return Ok(Bytes::new())
        }

        let (first, last) = (self.pos, self.pos + len as u64 - 1);
        let range = format!("bytes={first}-{last}");

        let request = self.bucket.http_client.request(Method::GET, self.bucket.url(&self.remote_path));
        let request: RequestBuilder = self.bucket.start_request(request);
        let request = request.header("range", range);
        let request = self.bucket.sign_request(request)?;
    
        debug!(request_full=?request);

        let result = self.bucket.http_client.execute(request).await.cx("Client::execute")?;
        if result.status().is_success() {
            if result.status().as_u16() != 206 { //Partial content
                return Err(CloudRuError::ReturningRangesNotSupported)
            }
            let rv: Bytes = result.bytes().await?;
            self.pos += rv.len() as u64;
            Ok(rv)
        } else {
            Err(CloudRuError::API(result.status(), result.text().await.cx("text")?))
        }

    }

    /// write/append `data` to the bucket 
    pub async fn write(&mut self, data: Bytes) -> Result<usize> {
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
        let write_op= self.fs_type.eval_write_op(self.pos, self.len)?;
        let url = write_op.modify_url(self.bucket.url(&self.remote_path));
        let data_len = data.len();

        let request = self.bucket.http_client.request(write_op.method, url)
            .header("Content-Length", format!("{data_len}"));
        let request: RequestBuilder = self.bucket.start_request(request);
        let request = request.body(data);
        let request = self.bucket.sign_request(request)?;

        debug!(request_full=?request);

        let result = self.bucket.http_client.execute(request).await.cx("Client::execute")?;
        bail_on_failure!(result);
        self.pos += data_len as u64;
        if self.pos > self.len { self.len = self.pos }
        Ok(data_len)
    }
}

impl io::Seek for ObjectIO {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            io::SeekFrom::Start(pos) =>
                pos,
            io::SeekFrom::Current(off) => 
                self.pos.checked_add_signed(off).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
            io::SeekFrom::End(off) => 
                self.len.checked_add_signed(off).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?,
        };
        Ok(self.pos)
    }
}