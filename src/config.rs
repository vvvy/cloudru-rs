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
    pub apig: Option<String>,
    pub fg: Option<String>,
    pub obs: Option<String>,
}

#[derive(Default)]
pub struct Config {
    pub endpoint: Endpoint,
    pub project_id: Option<String>,
    pub region: Option<String>
}

pub fn read_config(path: String) -> Result<Config> {
    let mut c = Config::default();

    let path = tildeexpand(path);
    let p = std::path::PathBuf::from(&path);
    if !p.exists() { return Ok(c) }

    let config_ini = ini::Ini::load_from_file(p).cxd(|| format!("reading config file {path}"))?;
    let ep = &config_ini["endpoint"];
    c.endpoint.apig = ep.get("apig").map(|s| s.to_owned());
    c.endpoint.fg = ep.get("fg").map(|s| s.to_owned());
    c.endpoint.obs = ep.get("obs").map(|s| s.to_owned());
    let ep = &config_ini["common"];
    c.project_id = ep.get("project_id").map(|s| s.to_owned());
    c.region = ep.get("region").map(|s| s.to_owned());
    Ok(c)
}
