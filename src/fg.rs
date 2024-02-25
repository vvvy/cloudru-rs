//!Function Graph-related api
use crate::*;

pub mod apdu;

pub struct FgClient {
    endpoint: String,
    project_id: String,
    aksk: AkSk,
}

impl FgClient {
    pub fn new(endpoint: String, project_id: String, aksk: AkSk) -> Self { Self { endpoint, project_id, aksk } }
    pub fn logging_to_lts_enable(&self) -> Result<JsonValue> { 
        logging_to_lts_enable(&self.endpoint, &self.project_id, &self.aksk) 
    }
    pub fn logging_to_lts_detail(&self, urn: &str) -> Result<JsonValue> {
        logging_to_lts_detail(&self.endpoint, &self.project_id, urn, &self.aksk)
    }
}


pub fn logging_to_lts_enable(
    fg_endpoint: &str,
    project_id: &str,
    aksk: &AkSk
) -> Result<JsonValue> {
    //POST /v2/{project_id}/fgs/functions/enable-lts-logs
    api_call!(POST /"{fg_endpoint}/v2/{project_id}/fgs/functions/enable-lts-logs" ;
        aksk
    )
}

pub fn logging_to_lts_detail(
    fg_endpoint: &str,
    project_id: &str,
    urn: &str,
    aksk: &AkSk
) -> Result<JsonValue> {
    //GET /v2/{project_id}/fgs/functions/{urn}/lts-log-detail
    api_call!(GET /"{fg_endpoint}/v2/{project_id}/fgs/functions/{urn}/lts-log-detail" ;
        aksk
    )
}
