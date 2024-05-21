use reqwest::Request;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use crate::Result;
use crate::shared::signing;

pub const X_SDK_DATE: &'static str = "X-Sdk-Date";

const LONG_DATETIME: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year][month][day]T[hour][minute][second]Z");

pub fn x_sdk_date(dt: time::OffsetDateTime) -> Result<String> {
    Ok(dt.format(LONG_DATETIME)?)
}

const HEXDIGITS: [char;16] = ['0' ,'1' ,'2' ,'3' ,'4' ,'5' ,'6' ,'7' ,'8' ,'9' ,'a' ,'b' ,'c' ,'d' ,'e' ,'f'];

pub fn hexencode(bytes: &[u8]) -> String {
    bytes.iter().flat_map(|b| [b.rotate_right(4) & 0x0f, b & 0x0f].into_iter()).map(|b| HEXDIGITS[b as usize]).collect()
}

#[test]
fn test_hexencode() {
    assert_eq!(&hexencode(&[0x12, 0xab, 0x0f, 0xf0]), "12ab0ff0");
}

pub fn canonical_request(m: &Request) -> Result<String> {
    //calculate body hash
    let b = m.body().and_then(|b| b.as_bytes()).unwrap_or(&[]);
    let mut hasher = Sha256::new();
    hasher.update(b);
    let body_hash = &hasher.finalize()[..];

    //add trailing / to url, if not in place
    let mut url = m.url().clone();
    if !url.path().ends_with('/') {
        let path = url.path().to_string() + "/";
        url.set_path(&path);
    }
    Ok(signing::canonical_request(m.method().as_str(), &url, m.headers(), &hexencode(body_hash))?)
}



pub fn string_to_sign(canonical_request: &str, dt: time::OffsetDateTime) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(canonical_request.as_bytes());
    let result = hasher.finalize();
    let crhash = hexencode(&result[..]);
    let dts = x_sdk_date(dt)?;
    Ok(format!("SDK-HMAC-SHA256\n{dts}\n{crhash}"))
}

pub fn signature(string_to_sign: &str, secret_key: &str) -> Result<String> {
    let mut hmac = Hmac::<sha2::Sha256>::new_from_slice(secret_key.as_bytes())?;
    hmac.update(string_to_sign.as_bytes());
    Ok(hexencode(&hmac.finalize().into_bytes()))
}

pub fn time_stamp_and_sign(r: &mut Request, dt: time::OffsetDateTime, ak: &str, sk: &str) -> Result<()> {
    let hs = r.headers_mut();
    let dts = x_sdk_date(dt)?;
    hs.insert(X_SDK_DATE, dts.parse()?);
    let cr = canonical_request(r)?;
    let sh = signing::signed_header_string(r.headers());    
    let s2s = string_to_sign(&cr, dt).unwrap();
    let sig = signature(&s2s, sk)?;
    // SDK-HMAC-SHA256 Access=QTWAOYTTINDUT2QVKYUC, SignedHeaders=content-type;host;x-sdk-date, Signature=7be6668032f70418fcc22abc52071e57aff61b84a1d2381bb430d6870f4f6ebe"
    let a11n = format!("SDK-HMAC-SHA256 Access={ak}, SignedHeaders={sh}, Signature={sig}");
    let hs = r.headers_mut();
    hs.insert("Authorization", a11n.parse()?);
    Ok(())
}

#[test]
fn test_canonical_request() {
    use reqwest::{Method, Url};
    /*
    GET https://service.region.example.com/v1/77b6a44cba5143ab91d13ab9a8ff44fd/vpcs?limit=2&marker=13551d6b-755d-4757-b956-536f674975c0 HTTP/1.1
Host: service.region.example.com
X-Sdk-Date: 20191115T033655Z
*/
    let u = "https://service.region.example.com/v1/77b6a44cba5143ab91d13ab9a8ff44fd/vpcs?limit=2&marker=13551d6b-755d-4757-b956-536f674975c0";
    let mut r = Request::new(Method::GET, Url::parse(u).unwrap());
    r.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
    r.headers_mut().insert("Host", "service.region.example.com".parse().unwrap());
    r.headers_mut().insert("X-Sdk-Date", "20191115T033655Z".parse().unwrap());

    let cr = canonical_request(&r).unwrap();
    println!("{}", cr);
    let dt = time::macros::datetime!(2019 - 11 - 15 03:36:55 utc);
    let s2s = string_to_sign(&cr, dt).unwrap();
    println!("{}", s2s);
    let sk = "MFyfvK41ba2giqM7Uio6PznpdUKGpownRZlmVmHc";
    let result = "7be6668032f70418fcc22abc52071e57aff61b84a1d2381bb430d6870f4f6ebe";
    assert_eq!(result, signature(&s2s, sk).unwrap());
}

#[test]
fn test_signature() {
    let s2s = "SDK-HMAC-SHA256\n20191115T033655Z\nb25362e603ee30f4f25e7858e8a7160fd36e803bb2dfe206278659d71a9bcd7a";
    let sk = "MFyfvK41ba2giqM7Uio6PznpdUKGpownRZlmVmHc";
    let result = "7be6668032f70418fcc22abc52071e57aff61b84a1d2381bb430d6870f4f6ebe";
    assert_eq!(result, signature(s2s, sk).unwrap());
}
