use http::HeaderMap;
use url::Url;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use crate::Result;

// https://perishablepress.com/stop-using-unsafe-characters-in-urls/
pub const FRAGMENT: &AsciiSet = &CONTROLS
    // URL_RESERVED
    .add(b':')
    .add(b'?')
    .add(b'#')
    .add(b'[')
    .add(b']')
    .add(b'@')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b';')
    .add(b'=')
    // URL_UNSAFE
    .add(b'"')
    .add(b' ')
    .add(b'<')
    .add(b'>')
    .add(b'%')
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'`');

pub const FRAGMENT_SLASH: &AsciiSet = &FRAGMENT.add(b'/');

/// Encode a URI following the specific requirements of the AWS service.
pub fn uri_encode(string: &str, encode_slash: bool) -> String {
    if encode_slash {
        utf8_percent_encode(string, FRAGMENT_SLASH).to_string()
    } else {
        utf8_percent_encode(string, FRAGMENT).to_string()
    }
}

/// Generate a canonical URI string from the given URL.
pub fn canonical_uri_string(uri: &Url) -> String {
    // decode `Url`'s percent-encoding and then reencode it
    // according to AWS's rules
    let decoded = percent_encoding::percent_decode_str(uri.path()).decode_utf8_lossy();
    uri_encode(&decoded, false)
}

/// Generate a canonical query string from the query pairs in the given URL.
pub fn canonical_query_string(uri: &Url) -> String {
    let mut keyvalues: Vec<(String, String)> = uri
        .query_pairs()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    keyvalues.sort();
    let keyvalues: Vec<String> = keyvalues
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                utf8_percent_encode(k, FRAGMENT_SLASH),
                utf8_percent_encode(v, FRAGMENT_SLASH)
            )
        })
        .collect();
    keyvalues.join("&")
}

/// Generate a canonical header string from the provided headers.
pub fn canonical_header_string(headers: &HeaderMap) -> Result<String> {
    let mut keyvalues = vec![];
    for (key, value) in headers.iter() {
        keyvalues.push(format!(
            "{}:{}",
            key.as_str().to_lowercase(),
            value.to_str()?.trim()
        ))
    }
    keyvalues.sort();
    Ok(keyvalues.join("\n"))
}

/// Generate a signed header string from the provided headers.
pub fn signed_header_string(headers: &HeaderMap) -> String {
    let mut keys = headers
        .keys()
        .map(|key| key.as_str().to_lowercase())
        .collect::<Vec<String>>();
    keys.sort();
    keys.join(";")
}


/// Generate a canonical request.
pub fn canonical_request(
    method: &str,
    url: &Url,
    headers: &HeaderMap,
    sha256: &str,
) -> Result<String> {
    Ok(format!(
        "{method}\n{uri}\n{query_string}\n{headers}\n\n{signed}\n{sha256}",
        method = method,
        uri = canonical_uri_string(url),
        query_string = canonical_query_string(url),
        headers = canonical_header_string(headers)?,
        signed = signed_header_string(headers),
        sha256 = sha256
    ))
}
