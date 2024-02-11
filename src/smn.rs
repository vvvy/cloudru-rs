use serde_derive::{Serialize, Deserialize};
use tracing::debug;
use crate::{Result, mauth, AkSk};

#[derive(Clone, Deserialize)]
pub struct SmnConfig {
    pub endpoint: String,
    //pub region: String,
}

pub struct SmnTopic {
    pub project_id: String,
    pub topic_urn: String
}

#[derive(Serialize)]
struct PublishApdu<'t> {
    subject: &'t str,
    message: &'t str
}


pub fn publish_message<V: serde::Serialize>(config: &SmnConfig, aksk: &AkSk, topic: &SmnTopic, subject: &str, value: &V
) -> Result<u16> {

    let url = format!("{e}/v2/{p}/notifications/topics/{t}/publish", 
        e=config.endpoint, p=topic.project_id, t=topic.topic_urn);

    let client = reqwest::blocking::Client::new();
        //reqwest::blocking::Client::builder().connect_timeout(Duration::from_secs(3)).build()?;
    debug!("Request: subject={subject}");
    let message = &serde_json::to_string(value)?;
    let apdu = PublishApdu { subject, message };
    let mut request = client.post(url).json(&apdu).build()?;
    let dt = time::OffsetDateTime::now_utc();
    mauth::time_stamp_and_sign(&mut request, dt, &aksk.ak, &aksk.sk)?;

    let resp = client.execute(request)?;
    let rv: u16 = resp.status().into();
    debug!("Response: {}", resp.text()?);

    Ok(rv)
}