use std::io;

use bytes::Bytes;

use tracing::{debug, instrument};
use reqwest::{header::{HeaderMap, HeaderValue}, Body, Method, Request, RequestBuilder};
use url::Url;

pub use crate::model::obs::*;
use self::error::ParameterKind;
use crate::shared::{urltools::WithVar, mauth_obs::*};
use super::*;
use crate::*;


pub struct ObsClient {
    endpoint: String,
    aksk: AkSk,
    http_client: HttpClient,
}

impl ObsClient {
    pub fn new(endpoint: String, aksk: AkSk, http_client: HttpClient) -> Self { Self { endpoint, http_client, aksk } }
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
            let err = $result.text().await.cx("error text in bail_on_failure")?;
            return Err(CloudRuInnerError::API(status, err).into());
        }        
    };
}


impl Bucket {
    pub fn new(bucket_name: String, obs_endpoint: String, aksk: AkSk, http_client: HttpClient) -> Result<Self> {
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
    pub async fn list_objects(&self, request: ListBucketRequest<'_>) -> Result<ListBucketResult> {
        
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

        let result = self.http_client.execute(request).await?;
        bail_on_failure!(result);

        let text = result.text().await?;
        debug!(response_text=?&text);
        let p = serde_xml_rs::from_str(&text)?;

        Ok(p)
    }

    pub async fn list(&self, prefix: Option<&str>) -> Result<ListBucketResult> {
        self.list_objects(ListBucketRequest { prefix, ..Default::default() }).await
    }

    fn start_request(&self, request: RequestBuilder) -> RequestBuilder {
        request.header("host", self.host.clone())
    }

    fn sign_request(&self, request: RequestBuilder) -> Result<Request> {
        request.timestamp_and_sign(&self.bucket_name, &self.aksk)
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

    /// create file-like IO object that track r/w position
    pub async fn object_io(&self, remote_path: impl AsRef<str>) -> Result<BucketAsyncIO> {
        let remote_path = remote_path.as_ref().to_string();

        // yield 0 if it is 404 error
        let len = match self.get_object_meta(&remote_path).await {
            Ok(metadata) => metadata.content_length.ok_or_else(|| CloudRuError::new(
                CloudRuInnerError::UnknownObjectLength, 
                remote_path.clone()
            ))?,
            Err(e) => match e.inner_ref() {
                CloudRuInnerError::API(n, _) if *n == 404 => Ok(0),
                _ => Err(e),
            }?
        };

        let bucket = self.clone();
        let pos = 0;
         
        Ok(BucketAsyncIO { remote_path, bucket, pos, len })
    }


}


pub struct BucketAsyncIO {
    remote_path: String,
    bucket: Bucket,
    pos: u64,
    len: u64,    
} 

impl BucketAsyncIO {
    /// Synchronizes cached position with the length of the actual object, so that we can resume appending to it.
    /// The object must exist and must be created in append mode.
    pub async fn sync_position(&mut self) -> Result<u64> {
        let meta = self.bucket.get_object_meta(&self.remote_path).await?;
        self.len = meta.content_length.ok_or_else(
            || CloudRuError::new(CloudRuInnerError::UnknownObjectLength, self.remote_path.to_owned())
        )?;
        if self.pos > self.len { self.pos = self.len }
        Ok(self.len)
    }

    pub fn pos(&self) -> u64 { self.pos }
    pub fn len(&self) -> u64 { self.len }

    /// read `len` bytes from the bucket 
    pub async fn read_bucket(&mut self, len: usize) -> Result<Bytes> {
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
                return Err(CloudRuInnerError::ReturningRangesNotSupported.into())
            }
            let rv: Bytes = result.bytes().await?;
            self.pos += rv.len() as u64;
            Ok(rv)
        } else {
            Err(CloudRuInnerError::API(result.status(), result.text().await.cx("text")?).into())
        }

    }

    /// write/append `data` to the bucket 
    pub async fn write_bucket(&mut self, data: Bytes) -> Result<usize> {
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
        let verb = if self.pos == self.len { "append" } else { "modify" };
        let data_len = data.len();

        let url = self.bucket.url(&self.remote_path)
            .with_var_key(verb)
            .with_var("position", &format!("{}", self.pos));

        let request = self.bucket.http_client.request(Method::POST, url);
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

impl io::Seek for BucketAsyncIO {
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