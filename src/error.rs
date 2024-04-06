use thiserror::Error;
use std::{fmt, io};
use std::result::Result;


#[derive(Debug)]
pub enum ParameterKind {
    S3BucketUrl
}

impl fmt::Display for ParameterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::S3BucketUrl => write!(f, "S3 bucket url")
        }
    }
}

#[derive(Error, Debug)]
pub enum CloudRuError {
    #[error("s3: {0}")]
    S3(#[from] s3::error::S3Error),

    #[error("s3: {0}")]
    S3cred(#[from] s3::creds::error::CredentialsError),

    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("reqwest: header value")]
    ReqwestHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("reqwest: header to string conversion")]
    ReqwestUrl(#[from] url::ParseError),

    #[error("reqwest: url parse")]
    ReqwestTostr(#[from] reqwest::header::ToStrError),

    #[error("HC API: code={0}, msg='{1}'")]
    API(reqwest::StatusCode, String),

    #[error("time conversion: {0}")]
    Time(#[from] time::error::Error),

    #[error("time format: {0}")]
    TimeFormat(#[from] time::error::Format),

    #[error("hmac: invalid length")]
    HMAC(#[from] hmac::digest::InvalidLength),
    
    #[error("json: ser/de: {0}")]
    Json(#[from] serde_json::error::Error),

    #[error("xml: ser/de: {0}")]
    Xml(#[from] serde_xml_rs::Error),

    #[error("fg_crt: request id not found")]
    RequestIdNotFound,

    #[error("fg_crt: invalid empty $RUNTIME_API_ADDR")]
    EmptyRuntimeAddr,

    #[error("parameter error: {0}")]
    Parameter(ParameterKind),

    #[error("ini: {0}")]
    Ini(#[from] ini::Error),

    #[error("UnresolvedEndpoint: svc={0}")]
    UnresolvedEndpoint(&'static str),

    #[error("UnknownObjectLength for {0}")]
    UnknownObjectLength(String),

    #[error("Missing project_id")]
    MissingProjectId,

    #[error("Returning ranges not supported")]
    ReturningRangesNotSupported,

    #[error("[{0}] {1}")]
    Context(String, Box<CloudRuError>),

    #[error("Other")]
    Other,
}

impl CloudRuError {
    pub fn cx(self, context: impl AsRef<str>) -> Self {
        let context = context.as_ref();
        if context.is_empty() {
            self
        } else {
            Self::Context(context.to_string(), Box::new(self))
        }
    }
}

pub trait Cx<T>  {
    fn cx(self, context: impl AsRef<str>) -> Result<T, CloudRuError>;
    fn cxd(self, context: impl FnOnce() -> String) -> Result<T, CloudRuError>;
}

impl<T, E> Cx<T> for Result<T, E> where CloudRuError: From<E>   {
    fn cx(self, context: impl AsRef<str>) -> Result<T, CloudRuError> {
        self.map_err(|e| Into::<CloudRuError>::into(e).cx(context))
    }

    fn cxd(self, context: impl FnOnce() -> String) -> Result<T, CloudRuError> {
        self.map_err(|e| Into::<CloudRuError>::into(e).cx(&context()))
    }
}

fn __test_as_error(e: Box<CloudRuError>) -> Box<dyn std::error::Error + Send + Sync> {
    e
}

fn __test_error_cx<T>(r: Result<T, s3::error::S3Error>) -> Result<T, CloudRuError> {
    r.cx("context")
}

impl From<CloudRuError> for io::Error {
    fn from(value: CloudRuError) -> Self {
        io::Error::new(io::ErrorKind::Other, value)
    }
}