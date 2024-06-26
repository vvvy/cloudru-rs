use std::sync::Arc;

use serde_derive::Serialize;
use super::*;
use crate::*;

pub struct ApigClient {
    endpoint: String,
    credentials: Credentials,
    http_client: Arc<HttpClient>,
}

impl ApigClient {
    pub fn new(endpoint: String, credentials: Credentials, http_client: Arc<HttpClient>) -> Self { Self { endpoint, credentials, http_client } }
    #[inline]
    pub fn add_certificate(&self, 
        group_id: &str, 
        domain_id: &str,
        cert_name: &str,
        cert_content: &str,
        private_key: &str,
    ) -> Result<JsonValue> {
        add_certificate(&self.endpoint, group_id, domain_id, cert_name, cert_content, private_key, &self.credentials, &self.http_client)
    }

    pub fn get_certificate(&self,
        group_id: &str, 
        domain_id: &str,
        cert_id: &str,
    ) -> Result<JsonValue> {
        get_certificate(&self.endpoint, group_id, domain_id, cert_id, &self.credentials, &self.http_client)
    }
    
    pub fn delete_certificate(&self,
        group_id: &str, 
        domain_id: &str,
        cert_id: &str,
    ) -> Result<JsonValue> {
        delete_certificate(&self.endpoint, group_id, domain_id, cert_id, &self.credentials, &self.http_client)
    }
    
    pub fn get_api_group_detail(&self, group_id: &str)  -> Result<JsonValue> {       
        get_api_group_detail(&self.endpoint, group_id, &self.credentials, &self.http_client)
    }
    
}


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
    credentials: &Credentials,
    client: &HttpClient,
) -> Result<JsonValue> {
    api_call!(POST /"{apig_endpoint}/v1.0/apigw/api-groups/{group_id}/domains/{domain_id}/certificate" ;
        &CertApdu{ name:cert_name, cert_content, private_key }, 
        credentials,
        client
    )
}

pub fn get_certificate(
    apig_endpoint: &str, 
    group_id: &str, 
    domain_id: &str,
    cert_id: &str,
    credentials: &Credentials,
    client: &HttpClient
) -> Result<JsonValue> {
    api_call!(GET /"{apig_endpoint}/v1.0/apigw/api-groups/{group_id}/domains/{domain_id}/certificate/{cert_id}"; credentials, client)
}

pub fn delete_certificate(
    apig_endpoint: &str, 
    group_id: &str, 
    domain_id: &str,
    cert_id: &str,
    credentials: &Credentials,
    client: &HttpClient
) -> Result<JsonValue> {
    api_call!(DELETE /"{apig_endpoint}/v1.0/apigw/api-groups/{group_id}/domains/{domain_id}/certificate/{cert_id}"; credentials, client)
}

pub fn get_api_group_detail(    
    apig_endpoint: &str, 
    group_id: &str, 
    credentials: &Credentials,
    client: &HttpClient)  -> Result<JsonValue> {       
    api_call!(GET /"{apig_endpoint}/v1.0/apigw/api-groups/{group_id}"; credentials, client)
}