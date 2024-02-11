use std::collections::HashMap;

use phf::phf_map;
use crate::*;

pub const DEFAULT_CREDENTIALS_FILE: &'static str = "~/.cloudru/credentials";
pub const DEFAULT_CONFIG_FILE: &'static str = "~/.cloudru/config";
pub const DEFAULT_CREDENTIAL: &'static str = "default";
pub const DEFAULT_LOG_LEVEL:  &'static str = "INFO";

pub const ACCESS_KEY_ID_KEY: &'static str = "access_key_id";
pub const SECRET_ACCESS_KEY_KEY: &'static str = "secret_access_key";



fn tildeexpand(s: String) -> String {
    let v = std::env::var("HOME").unwrap_or("~".to_owned());
    s.replace("~", &v)
}

pub fn read_credentials(path: String, id: String) -> Result<AkSk> {
    let path = tildeexpand(path);
    let cred_ini = ini::Ini::load_from_file(&path).cxd(|| format!("reading credentials file {path}"))?;
    let def = &cred_ini[id.as_ref()];
    Ok(AkSk{ ak: def[ACCESS_KEY_ID_KEY].to_owned(), sk:  def[SECRET_ACCESS_KEY_KEY].to_owned() })
}

#[derive(Default)]
pub struct Endpoint {
    endpoint: HashMap<String, String>
}

impl Endpoint {
    pub fn resolve<'t>(&'t self, service_id: &'static str, endpoint: Option<&'t str>) -> Result<&'t str> {
        if let Some(e) = endpoint { return Ok(e); }
        if let Some(e) = self.endpoint.get(service_id).map(|s| s.as_str()) { return Ok(e); }
        if let Some(e) =  DEFAULT_ENDPOINTS.get(service_id).map(|&e| e) { return Ok(e); }
        Err(CloudRuInnerError::UnresolvedEndpoint(service_id).into())
    }
}

#[derive(Default)]
pub struct Config {
    pub endpoint: Endpoint,
    pub project_id: Option<String>,
    pub region: String
}

pub fn read_config(path: String) -> Result<Config> {
    let mut c = Config::default();

    let path = tildeexpand(path);
    let p = std::path::PathBuf::from(&path);
    if !p.exists() { return Ok(c) }

    let config_ini = ini::Ini::load_from_file(p).cxd(|| format!("reading config file {path}"))?;

    let endpoint = &config_ini["endpoint"];
    c.endpoint = Endpoint { endpoint: endpoint.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() };

    let common = &config_ini["common"];
    c.project_id = common.get("project_id").map(|s| s.to_owned());
    c.region = common.get("region").unwrap_or(DEFAULT_REGION).to_string();
    Ok(c)
}

#[allow(non_upper_case_globals)]
pub mod svc_id {
    pub static apig: &str = "apig";
    pub static fg: &str = "fg";
    pub static obs: &str = "obs";
}

pub const DEFAULT_ENDPOINTS: phf::Map<&'static str, &'static str> = phf_map!{
    "apig" => "https://apig.ru-moscow-1.hc.sbercloud.ru",
    "fg" => "https://functiongraph.ru-moscow-1.hc.sbercloud.ru",
    "obs" => "https://obs.ru-moscow-1.hc.sbercloud.ru",
};


pub const DEFAULT_REGION: &'static str = "ru-moscow-1";