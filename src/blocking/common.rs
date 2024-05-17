use tracing::debug;
use reqwest::{Method, blocking::Request};

use super::mauth;
use super::*;
use crate::*;


macro_rules! api_call {
    (GET $url:expr, $credentials:expr, $client:expr) => { 
        crate::blocking::common::auth_api_call_noq(reqwest::Method::GET, $url, $credentials, $client) 
    };
    (GET / $($url:tt),+ ; $credentials:expr, $client:expr) => { 
        crate::blocking::common::auth_api_call_noq(reqwest::Method::GET, &format!($($url),+), $credentials, $client) 
    };
    (POST $url:expr, $q:expr, $credentials:expr, $client:expr) => { 
        crate::blocking::common::auth_api_call(reqwest::Method::POST, $url, $q, $credentials, $client) 
    };
    (POST / $($url:tt),+ ; $q:expr, $credentials:expr, $client:expr) => { 
        crate::blocking::common::auth_api_call(reqwest::Method::POST, &format!($($url),+), $q, $credentials, $client) 
    };
    (POST $url:expr, $credentials:expr, $client:expr) => { 
        crate::blocking::common::auth_api_call_noq(Method::POST, $url, $credentials, $client) 
    };
    (POST / $($url:tt),+ ; $credentials:expr, $client:expr) => { 
        crate::blocking::common::auth_api_call_noq(reqwest::Method::POST, &format!($($url),+), $credentials, $client) 
    };
    (DELETE $url:expr, $credentials:expr, $client:expr) => { 
        crate::blocking::common::auth_api_call_noq(reqwest::Method::DELETE, $url, $credentials, $client) 
    };
    (DELETE / $($url:tt),+ ; $credentials:expr, $client:expr) => { 
        crate::blocking::common::auth_api_call_noq(reqwest::Method::DELETE, &format!($($url),+), $credentials, $client) 
    };
}

/// Authenticated API call
pub fn auth_api_call_explicit<R: for<'d> serde::Deserialize<'d> + Default>(
    mut request: Request, 
    credentials: &Credentials,
    client: &HttpClient,
) -> Result<R> {
    let dt = time::OffsetDateTime::now_utc();
    mauth::time_stamp_and_sign(&mut request, dt, &credentials.ak, &credentials.sk)?;
    debug!("Request-Full: {request:?}");
    let resp = client.execute(request)?;
    let status = resp.status();
    debug!("Response: status={} len={:?}", status, resp.content_length());
    match status {
        reqwest::StatusCode::NO_CONTENT => Ok(R::default()),
        s if s.is_success() => Ok(resp.json()?),
        s => Err(CloudRuError::API(s, resp.text()?))
    }
}

/// Authenticated API call
pub fn auth_api_call<R: for<'d> serde::Deserialize<'d> + Default, Q: serde::Serialize>(
    m: Method, url: &str, q: &Q, credentials: &Credentials, client: &HttpClient
) -> Result<R> {
    debug!("Request: {m} {url}");
    let r = client.request(m, url).json(q).build()?;
    auth_api_call_explicit(r, credentials, client)
}

/// Authenticated API call w/o request body
pub fn auth_api_call_noq<R: for<'d> serde::Deserialize<'d> + Default>(m: Method, url: &str, credentials: &Credentials, client: &HttpClient) -> Result<R> {
    debug!("Request: {m} {url}");
    let r = client.request(m, url).build()?;
    auth_api_call_explicit(r, credentials, client)
}