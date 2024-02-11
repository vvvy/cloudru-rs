use thiserror::Error;
use std::fmt;
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
pub enum HCInnerError {
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
}

#[derive(Error, Debug)]
pub struct HCError {
    inner: HCInnerError,
    context: String
}

impl fmt::Display for HCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error: {}", self.inner)?;
        if !self.context.is_empty() {
            writeln!(f, "Context: {}", self.context)?;
        }
        Ok(())
    }
}

impl HCError {
    pub fn cx(self, context: impl AsRef<str>) -> Self {
        let context = if self.context.is_empty() {
            context.as_ref().to_owned()
        } else {
            self.context + "/" + context.as_ref()
        };
        Self { context, ..self }
    }
    pub fn inner_ref(&self) -> &HCInnerError { &self.inner }
}

impl<E> From<E> for HCError where HCInnerError: From<E> {
    fn from(value: E) -> Self {
        Self { inner: From::from(value), context: Default::default() }
    }
}


pub trait Cx<T>  {
    fn cx(self, context: impl AsRef<str>) -> Result<T, HCError>;
    fn cxd(self, context: impl FnOnce() -> String) -> Result<T, HCError>;
}

impl<T, E> Cx<T> for Result<T, E> where HCError: From<E>   {
    fn cx(self, context: impl AsRef<str>) -> Result<T, HCError> {
        self.map_err(|e| Into::<HCError>::into(e).cx(context))
    }

    fn cxd(self, context: impl FnOnce() -> String) -> Result<T, HCError> {
        self.map_err(|e| Into::<HCError>::into(e).cx(&context()))
    }
}

fn __test_as_error(e: Box<HCError>) -> Box<dyn std::error::Error> {
    e
}

fn __test_error_cx<T>(r: Result<T, s3::error::S3Error>) -> Result<T, HCError> {
    r.cx("context")
}