use hmac::{Hmac, Mac};
use base64::{Engine as _, engine::general_purpose};
use reqwest::blocking::Request;
use phf::phf_set;
use tracing::trace;
use crate::*;

pub const STANDARD_DATE: &'static str = "date";

const STD_DATETIME: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT");

pub fn obs_date(dt: time::OffsetDateTime) -> Result<String> {
    Ok(dt.to_offset(time::UtcOffset::UTC).format(STD_DATETIME)?)
}

const SUBRESOURCES: phf::Set<&str> = phf_set!(
    "CDNNotifyConfiguration", 
    "acl", "append", "attname",  "cors", "customdomain", "delete",
    "deletebucket", "encryption", "length", "lifecycle", "location", "logging",
    "metadata", "modify", "name", "notification", "partNumber", "policy", "position", "quota",
    "rename", "replication", "response-cache-control", "response-content-disposition",
    "response-content-encoding", "response-content-language", "response-content-type", "response-expires",
    "restore", " storageClass", "storagePolicy", "storageinfo", "torrent", "truncate",
    "uploadId", "uploads", "versionId", "versioning", "versions", "website", 
    "x-obs-security-token");

pub fn string_to_sign(bucket_name: &str, m: &Request) -> Result<String> {
/*  HTTP-Verb + "\n" + 
    Content-MD5 + "\n" + 
    Content-Type + "\n" + 
    Date + "\n" + 
    CanonicalizedHeaders + CanonicalizedResource */
    fn get_hv<'t>(m: &'t Request, h: &str) -> Result<&'t str> {
        Ok(match m.headers().get(h) { Some(hv) => hv.to_str()?, None => "" })
    }

    let verb = m.method().to_string();
    let content_md5 = get_hv(m, "content-md5")?;
    let content_type = get_hv(m, "content-type")?;
    let date = get_hv(m, "date")?;

    let hv: std::result::Result<String, _> = m.headers().iter()
        .filter(|(hn, _)| hn.as_str().starts_with("x-obs-"))
        .map(|(hn, hv)| hv.to_str().map(|hv| format!("{}:{}\n", hn.as_str(), hv)))
        .collect();
    let canonicalized_headers = hv?;

    let mut canonicalized_resource = format!("/{}{}", bucket_name, m.url().path());
    let kvm: std::collections::BTreeMap<String, String> =
        m.url().query_pairs()
            .filter_map(|(k, v)| if SUBRESOURCES.contains(k.as_ref()) { Some((k.to_string(), v.to_string())) } else { None })
            .collect();
    if !kvm.is_empty() {
        canonicalized_resource.push('?');
        let v: Vec<String> = kvm.into_iter().map(|(k, v)| if v.is_empty() { k } else { k + "=" + &v }).collect();
        canonicalized_resource += &v.join("&");
    }

    Ok(format!("{verb}\n{content_md5}\n{content_type}\n{date}\n{canonicalized_headers}{canonicalized_resource}"))
}

pub fn signature(string_to_sign: &str, secret_key: &str) -> Result<String> {
    let mut hmac = Hmac::<sha1::Sha1>::new_from_slice(secret_key.as_bytes())?;
    hmac.update(string_to_sign.as_bytes());
    let hmac_binary: &[u8] = &hmac.finalize().into_bytes();
    let encoded = general_purpose::STANDARD.encode(hmac_binary);
    Ok(encoded)
}

pub fn time_stamp_and_sign(bucket_name: &str, r: &mut Request, dt: time::OffsetDateTime, ak: &str, sk: &str) -> Result<()> {
    let hs = r.headers_mut();

    let dts = obs_date(dt)?;
    hs.insert(STANDARD_DATE, dts.parse()?);
    
    let s2sign = string_to_sign(bucket_name, r)?;
    trace!("s2sign:`{s2sign}`");
    let sig = signature(&s2sign, sk)?;

    // Authorization: OBS AccessKeyID:signature
    let a11n = format!("OBS {ak}:{sig}");
    let hs = r.headers_mut();
    hs.insert("Authorization", a11n.parse()?);
    Ok(())
}


#[test]
fn test_string_to_sign() {
    macro_rules! req {
        ($meth:expr, $url:expr, $($h:expr => $v:expr),*) => { {
            let url = reqwest::Url::parse($url).unwrap();
            let mut req = Request::new($meth, url);
            $(
                req.headers_mut().insert($h, $v.parse().unwrap());
            )*
            req
        } };
    }

    use reqwest::Method;
    /*
    GET /object.txt HTTP/1.1
    Host: bucket.obs.region.example.com
    Date: Sat, 12 Oct 2015 08:12:38 GMT
    --------
    GET \n
    \n
    \n
    Sat, 12 Oct 2015 08:12:38 GMT\n
    /bucket/object.txt
    */
    let req = req!(Method::GET, "https://bucket.endpoint/object.txt",
        "host" => "bucket.obs.region.example.com",
        "date" => "Sat, 12 Oct 2015 08:12:38 GMT"
    );
    let expected = "GET\n\n\nSat, 12 Oct 2015 08:12:38 GMT\n/bucket/object.txt";
    assert_eq!(expected, string_to_sign("bucket", &req).unwrap());

    /*
    PUT /object.txt HTTP/1.1
    User-Agent: curl/7.15.5
    Host: bucket.obs.region.example.com
    x-obs-date:Tue, 15 Oct 2015 07:20:09 GMT
    x-obs-security-token: YwkaRTbdY8g7q....
    content-type: text/plain
    Content-Length: 5913339
    --------
    PUT\n
    \n
    text/plain\n
    \n
    x-obs-date:Tue, 15 Oct 2015 07:20:09 GMT\n
    x-obs-security-token:YwkaRTbdY8g7q....\n
    /bucket/object.txt
     */
    let req = req!(Method::PUT, "https://bucket.endpoint/object.txt",
        "User-Agent" => "curl/7.15.5",
        "Host" => "bucket.obs.region.example.com",
        "x-obs-date" => "Tue, 15 Oct 2015 07:20:09 GMT",
        "x-obs-security-token" => "YwkaRTbdY8g7q....",
        "content-type" => "text/plain",
        "Content-Length" => "5913339"
    );
    let expected = 
    "PUT\n\ntext/plain\n\nx-obs-date:Tue, 15 Oct 2015 07:20:09 GMT\nx-obs-security-token:YwkaRTbdY8g7q....\n/bucket/object.txt";
    assert_eq!(expected, string_to_sign("bucket", &req).unwrap());

    /*
    PUT /object.txt HTTP/1.1
    User-Agent: curl/7.15.5
    Host: bucket.obs.region.example.com
    Date: Mon, 14 Oct 2015 12:08:34 GMT
    x-obs-acl: public-read
    content-type: text/plain
    Content-Length: 5913339
    ----------
    PUT\n
    \n
    text/plain\n
    Mon, 14 Oct 2015 12:08:34 GMT\n
    x-obs-acl:public-read\n
    /bucket/object.txt
    */
    let req = req!(Method::PUT, "https://bucket.endpoint/object.txt",
        "User-Agent" => "curl/7.15.5",
        "Host" => "bucket.obs.region.example.com",
        "Date" => "Mon, 14 Oct 2015 12:08:34 GMT",
        "x-obs-acl" => "public-read",
        "content-type" => "text/plain",
        "Content-Length" => "5913339"
    );
    let expected = "PUT\n\ntext/plain\nMon, 14 Oct 2015 12:08:34 GMT\nx-obs-acl:public-read\n/bucket/object.txt";
    assert_eq!(expected, string_to_sign("bucket", &req).unwrap());

    /*
    GET /object.txt?acl HTTP/1.1
    Host: bucket.obs.region.example.com
    Date: Sat, 12 Oct 2015 08:12:38 GMT
    ---------
    GET \n
    \n
    \n
    Sat, 12 Oct 2015 08:12:38 GMT\n
    /bucket/object.txt?acl
    */
    let req = req!(Method::GET, "https://bucket.endpoint/object.txt?acl",
        "Host" => "bucket.obs.region.example.com",
        "Date" => "Sat, 12 Oct 2015 08:12:38 GMT"
    );
    let expected = "GET\n\n\nSat, 12 Oct 2015 08:12:38 GMT\n/bucket/object.txt?acl";
    assert_eq!(expected, string_to_sign("bucket", &req).unwrap());

    /*
    PUT /object.txt HTTP/1.1
    Host: bucket.obs.region.example.com
    x-obs-date:Tue, 15 Oct 2015 07:20:09 GMT
    Content-MD5: I5pU0r4+sgO9Emgl1KMQUg==
    Content-Length: 5913339
    --------------
    PUT\n
    I5pU0r4+sgO9Emgl1KMQUg==\n
    \n
    \n
    x-obs-date:Tue, 15 Oct 2015 07:20:09 GMT\n
    /bucket/object.txt
    */
    let req = req!(Method::PUT, "https://bucket.endpoint/object.txt",
        "Host" => "bucket.obs.region.example.com",
        "x-obs-date" => "Tue, 15 Oct 2015 07:20:09 GMT",
        "Content-MD5" => "I5pU0r4+sgO9Emgl1KMQUg==",
        "Content-Length" => "5913339"
    );
    let expected = "PUT\nI5pU0r4+sgO9Emgl1KMQUg==\n\n\nx-obs-date:Tue, 15 Oct 2015 07:20:09 GMT\n/bucket/object.txt";
    assert_eq!(expected, string_to_sign("bucket", &req).unwrap());

/*
PUT /object.txt HTTP/1.1
Host: obs.ccc.com
x-obs-date:Tue, 15 Oct 2015 07:20:09 GMT
Content-MD5: I5pU0r4+sgO9Emgl1KMQUg==
Content-Length: 5913339
------------
PUT\n
I5pU0r4+sgO9Emgl1KMQUg==\n
\n
\n
x-obs-date:Tue, 15 Oct 2015 07:20:09 GMT\n
/obs.ccc.com/object.txt
*/
    let req = req!(Method::PUT, "https://bucket.endpoint/object.txt",
        "Host" => "obs.ccc.com",
        "x-obs-date" => "Tue, 15 Oct 2015 07:20:09 GMT",
        "Content-MD5" => "I5pU0r4+sgO9Emgl1KMQUg==",
        "Content-Length" => "5913339"
    );
    let expected = "PUT\nI5pU0r4+sgO9Emgl1KMQUg==\n\n\nx-obs-date:Tue, 15 Oct 2015 07:20:09 GMT\n/obs.ccc.com/object.txt";
    assert_eq!(expected, string_to_sign("obs.ccc.com", &req).unwrap());
}