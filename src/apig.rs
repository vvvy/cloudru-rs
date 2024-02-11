use serde_derive::Serialize;
use crate::*;

#[derive(Serialize)]
pub struct CertApdu<'t> {
    name: &'t str,
    cert_content: &'t str,
    private_key: &'t str
}

pub fn add_certificate(
    apig_endpoint: &str, 
    group_id: &str, 
    domain_id: &str,
    cert_name: &str,
    cert_content: &str,
    private_key: &str,
    aksk: &AkSk
) -> Result<JsonValue> {
    api_call!(POST /"{apig_endpoint}/v1.0/apigw/api-groups/{group_id}/domains/{domain_id}/certificate" ;
        &CertApdu{ name:cert_name, cert_content, private_key }, 
        aksk
    )
}

pub fn get_certificate(
    apig_endpoint: &str, 
    group_id: &str, 
    domain_id: &str,
    cert_id: &str,
    aksk: &AkSk
) -> Result<JsonValue> {
    api_call!(GET /"{apig_endpoint}/v1.0/apigw/api-groups/{group_id}/domains/{domain_id}/certificate/{cert_id}"; aksk)
}

pub fn delete_certificate(
    apig_endpoint: &str, 
    group_id: &str, 
    domain_id: &str,
    cert_id: &str,
    aksk: &AkSk
) -> Result<JsonValue> {
    api_call!(DELETE /"{apig_endpoint}/v1.0/apigw/api-groups/{group_id}/domains/{domain_id}/certificate/{cert_id}"; aksk)
}

pub fn get_api_group_detail(    
    apig_endpoint: &str, 
    group_id: &str, 
    aksk: &AkSk)  -> Result<JsonValue> {       
    api_call!(GET /"{apig_endpoint}/v1.0/apigw/api-groups/{group_id}"; aksk)
}