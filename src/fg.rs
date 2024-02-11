//!Function Graph-related api
use crate::*;

pub mod apdu;

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
