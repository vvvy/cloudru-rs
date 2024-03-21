use tracing::debug;
use crate::{Result, mauth, AkSk, CloudRuInnerError, HttpClient};
use reqwest::{Method, blocking::Request};


macro_rules! api_call {
    (GET $url:expr, $aksk:expr, $client:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::GET, $url, $aksk, $client) 
    };
    (GET / $($url:tt),+ ; $aksk:expr, $client:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::GET, &format!($($url),+), $aksk, $client) 
    };
    (POST $url:expr, $q:expr, $aksk:expr, $client:expr) => { 
        crate::common::auth_api_call(reqwest::Method::POST, $url, $q, $aksk, $client) 
    };
    (POST / $($url:tt),+ ; $q:expr, $aksk:expr, $client:expr) => { 
        crate::common::auth_api_call(reqwest::Method::POST, &format!($($url),+), $q, $aksk, $client) 
    };
    (POST $url:expr, $aksk:expr, $client:expr) => { 
        crate::common::auth_api_call_noq(Method::POST, $url, $aksk, $client) 
    };
    (POST / $($url:tt),+ ; $aksk:expr, $client:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::POST, &format!($($url),+), $aksk, $client) 
    };
    (DELETE $url:expr, $aksk:expr, $client:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::DELETE, $url, $aksk, $client) 
    };
    (DELETE / $($url:tt),+ ; $aksk:expr, $client:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::DELETE, &format!($($url),+), $aksk, $client) 
    };
}

/// Authenticated API call
pub fn auth_api_call_explicit<R: for<'d> serde::Deserialize<'d> + Default>(
    mut request: Request, 
    aksk: &AkSk,
    client: &HttpClient,
) -> Result<R> {
    let dt = time::OffsetDateTime::now_utc();
    mauth::time_stamp_and_sign(&mut request, dt, &aksk.ak, &aksk.sk)?;
    debug!("Request-Full: {request:?}");
    let resp = client.execute(request)?;
    let status = resp.status();
    debug!("Response: status={} len={:?}", status, resp.content_length());
    match status {
        reqwest::StatusCode::NO_CONTENT => Ok(R::default()),
        s if s.is_success() => Ok(resp.json()?),
        s => Err(CloudRuInnerError::API(s, resp.text()?).into())
    }
}

/// Authenticated API call
pub fn auth_api_call<R: for<'d> serde::Deserialize<'d> + Default, Q: serde::Serialize>(
    m: Method, url: &str, q: &Q, aksk: &AkSk, client: &HttpClient
) -> Result<R> {
    debug!("Request: {m} {url}");
    let r = client.request(m, url).json(q).build()?;
    auth_api_call_explicit(r, aksk, client)
}

/// Authenticated API call w/o request body
pub fn auth_api_call_noq<R: for<'d> serde::Deserialize<'d> + Default>(m: Method, url: &str, aksk: &AkSk, client: &HttpClient) -> Result<R> {
    debug!("Request: {m} {url}");
    let r = client.request(m, url).build()?;
    auth_api_call_explicit(r, aksk, client)
}