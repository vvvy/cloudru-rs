use serde_derive::Deserialize;
use serde_json::Value;

/*
Incoming request, 
f=zw_tr:zwTestRust:latest, 
id=a9fd76c7-cf60-4ace-8213-3d8c55f023e2, 
data={
    "record":[
        {
            "event_source":"smn",
            "event_subscription_urn":"urn:fss:ru-moscow-1:0c257d1b5d8026de2f2ac01dba7dce47:function:zw_tr:zwTestRust:latest",
            "event_version":"1.0",
            "smn":{
                "message":"\"abc\"",
                "message_attributes":null,
                "message_id":"cada64e627734df8b1316666ec4d59a3",
                "subject":"abc",
                "timestamp":"2022-04-23T14:36:26Z",
                "topic_urn":"urn:smn:ru-moscow-1:0c257d1b5d8026de2f2ac01dba7dce47:zwEvent_tr",
                "type":"notification"
            }
        }
    ]
}
*/
//Notification as seen by a Function
#[derive(Deserialize)]
pub struct FnNotification {
    pub record: Vec<EventRecord>
}

#[derive(Deserialize)]
pub struct EventRecord {
    pub event_source: Option<String>,
    pub event_subscription_urn: Option<String>,
    pub event_version: Option<String>,
    pub smn: Option<SmnEvent>
}

#[derive(Deserialize)]
pub struct SmnEvent {
    pub message: Option<String>,
    pub message_attributes: Option<Value>,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub timestamp: Option<String>,
    pub topic_urn: Option<String>,
    #[serde(rename="type")]
    pub type_: Option<String>
}