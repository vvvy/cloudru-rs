use tracing::debug;
use crate::{Result, mauth, AkSk, CloudRuInnerError};
use reqwest::{Method, blocking::{Request, Client}};


macro_rules! api_call {
    (GET $url:expr, $aksk:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::GET, $url, $aksk) 
    };
    (GET / $($url:tt),+ ; $aksk:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::GET, &format!($($url),+), $aksk) 
    };
    (POST $url:expr, $q:expr, $aksk:expr) => { 
        crate::common::auth_api_call(reqwest::Method::POST, $url, $q, $aksk) 
    };
    (POST / $($url:tt),+ ; $q:expr, $aksk:expr) => { 
        crate::common::auth_api_call(reqwest::Method::POST, &format!($($url),+), $q, $aksk) 
    };
    (POST $url:expr, $aksk:expr) => { 
        crate::common::auth_api_call_noq(Method::POST, $url, $aksk) 
    };
    (POST / $($url:tt),+ ; $aksk:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::POST, &format!($($url),+), $aksk) 
    };
    (DELETE $url:expr, $aksk:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::DELETE, $url, $aksk) 
    };
    (DELETE / $($url:tt),+ ; $aksk:expr) => { 
        crate::common::auth_api_call_noq(reqwest::Method::DELETE, &format!($($url),+), $aksk) 
    };
}

/// Authenticated API call
pub fn auth_api_call_explicit<R: for<'d> serde::Deserialize<'d> + Default>(
    client: &Client,
    mut request: Request, 
    aksk: &AkSk
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
    m: Method, url: &str, q: &Q, aksk: &AkSk
) -> Result<R> {
    let client = Client::new();
    debug!("Request: {m} {url}");
    let r = client.request(m, url).json(q).build()?;
    auth_api_call_explicit(&client, r, aksk)
}

/// Authenticated API call w/o request body
pub fn auth_api_call_noq<R: for<'d> serde::Deserialize<'d> + Default>(m: Method, url: &str, aksk: &AkSk) -> Result<R> {
    let client = Client::new();
    debug!("Request: {m} {url}");
    let r = client.request(m, url).build()?;
    auth_api_call_explicit(&client, r, aksk)
}