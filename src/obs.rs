use std::io::Write;

use crate::*;
use reqwest::{Url, blocking::{Request, Client, Body, RequestBuilder}, Method, header::HeaderValue};
use error::ParameterKind;
use mauth_obs::*;
use serde_derive::Deserialize;
use tracing::{debug, instrument, Level, enabled};

#[derive(Debug)]
pub struct Bucket {
    bucket_name: String,
    bucket_url: Url,
    host: HeaderValue,
    aksk: AkSk
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
    fn with_var<T: AsRef<str> + ?Sized>(self, var: &str, val: &T) -> Url;
    fn with_var_opt<T: AsRef<str> + ?Sized>(self, var: &str, val: Option<&T>) -> Url;
}

impl WithVar for Url {
    fn with_var_opt<T: AsRef<str> + ?Sized>(mut self, var: &str, val: Option<&T>) -> Url {
        if let Some(val) = val {
            self.query_pairs_mut().append_pair(var, val.as_ref());
        }
        self
    }

    fn with_var<T: AsRef<str> + ?Sized>(mut self, var: &str, val: &T) -> Url {
        self.query_pairs_mut().append_pair(var, val.as_ref());
        self
    }
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
    pub key_count: u64,
    #[serde(rename="MaxKeys")]
    pub max_keys: u64,
    #[serde(rename="IsTruncated")]
    pub is_truncated: bool,
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
    #[serde(rename="Size")]
    pub size: u64,
    #[serde(rename="StorageClass")]
    pub storage_class: String,
}


impl Bucket {
    pub fn new(bucket_name: String, obs_endpoint: String, aksk: AkSk) -> Result<Self> {
        let mut bucket_url: Url = obs_endpoint.parse()?;
        let bucket_host = bucket_url.host()
            .ok_or(HCInnerError::Parameter(ParameterKind::S3BucketUrl))?;
        let host = format!("{}.{}", bucket_name, bucket_host);
        bucket_url.set_host(Some(&host))?;
        let host = host.parse()?;
        Ok(Self { bucket_name, bucket_url, host, aksk })
    }

    #[inline]
    fn url(&self, path: &str) -> Url {
        let mut url = self.bucket_url.clone();
        url.set_path(path);
        url
    }

    #[instrument]
    pub fn list(&self, prefix: Option<&str>) -> Result<ListBucketResult> {
        
        let client = Client::new();

        let url = self.url("/")
            .with_var("list-type", "2")
            .with_var_opt("prefix", prefix);

        let request = client.request(Method::GET, url)
            .header("host", self.host.clone())
            .timestamp_and_sign(&self.bucket_name, &self.aksk)?;

        debug!(request_full=?request);

        let result = client.execute(request)?;
        if result.status().is_success() {
            let p: ListBucketResult = if enabled!(Level::DEBUG) {
                let text = result.text()?;
                debug!(response_text=?&text);
                serde_xml_rs::from_str(&text)?
            } else {
                serde_xml_rs::from_reader(result)?
            };
            Ok(p)
        } else {
            Err(HCInnerError::API(result.status(), result.text()?).into())
        }
    }

    pub fn get_object<W: Write>(&self, remote_path: &str, w: &mut W) -> Result<()> {
        let client = Client::new();

        let request = client.request(Method::GET, self.url(remote_path))
            .header("host", self.host.clone())
            .timestamp_and_sign(&self.bucket_name, &self.aksk)?;
        
        debug!(request_full=?request);

        let mut result = client.execute(request)?;
        if result.status().is_success() {
            result.copy_to(w)?;
            Ok(())
        } else {
            Err(HCInnerError::API(result.status(), result.text()?).into())
        }
    }

    pub fn put_object<I>(&self, remote_path: &str, input: I) -> Result<()> where Body: From<I> {
        let client = Client::new();

        let request = client.request(Method::PUT, self.url(remote_path))
            .header("host", self.host.clone())
            .body(input)
            .timestamp_and_sign(&self.bucket_name, &self.aksk)?;

        debug!(request_full=?request);

        let result = client.execute(request)?;
        if result.status().is_success() {
            Ok(())
        } else {
            Err(HCInnerError::API(result.status(), result.text()?).into())
        }
    }

}