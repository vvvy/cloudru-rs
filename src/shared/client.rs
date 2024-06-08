use super::config::*;
use crate::*;

pub(crate) type ServiceId = &'static str;

pub struct Client<HC> {
    pub(crate) config: Config,
    pub(crate) credentials: Credentials,
    pub(crate) http_client: HC,
}

impl<HC> Client<HC> {
    pub fn builder() -> ClientBuilder { ClientBuilder::new() }

    #[inline]
    pub(crate) fn resolve_endpoint(&self, service_id: ServiceId) -> Result<String> {
        Ok(self.config.endpoint.resolve(service_id, None)?.to_string())
    }

    #[inline]
    pub(crate) fn resolve_project_id(&self) -> Result<String> {
        if let Some(project_id) = &self.config.project_id {
            Ok(project_id.clone())
        } else {
            Err(crate::error::CloudRuError::MissingProjectId)
        }
    }
}


#[derive(Debug, Clone, Default)]
pub struct ClientBuilder {
    pub config_file: Option<String>,
    pub credentials_file: Option<String>,
    pub credentials_id: Option<String>,
    pub project_id: Option<String>,
    pub region: Option<String>,
    pub credentials: Option<Credentials>,
}

impl ClientBuilder {
    pub fn new() -> Self { Self::default() }
    pub fn config_file(self, arg: &str) -> Self { Self { config_file: Some(arg.to_owned()), ..self } }
    pub fn credentials_file(self, arg: &str) -> Self { Self { credentials_file: Some(arg.to_owned()), ..self } }
    pub fn credentials_id(self, arg: &str) -> Self { Self { credentials_id: Some(arg.to_owned()), ..self } }
    pub fn project_id(self, arg: &str) -> Self { Self { project_id: Some(arg.to_owned()), ..self } }
    pub fn region(self, arg: &str) -> Self { Self { region: Some(arg.to_owned()), ..self } }
    pub fn credentials(self, arg: Credentials) -> Self { Self { credentials: Some(arg), ..self } }
    pub fn build_with_http_client<HC>(self, http_client: HC) -> Result<Client<HC>> {
        let (config_path, force) = self.config_file
            .map(|f| (f, true))
            .unwrap_or_else(|| (DEFAULT_CONFIG_FILE.to_owned(), false));

        let mut config = read_config(config_path, force)?;
        //config.project_id = self.project_id.or(config.project_id);
        if let Some(project_id) = self.project_id { config.project_id = Some(project_id); }
        if let Some(region) = self.region { config.region = region; }
        
        let credentials = if let Some(credentials) = self.credentials {
            credentials
        } else {
            read_credentials(
                self.credentials_file.unwrap_or_else(|| DEFAULT_CREDENTIALS_FILE.to_owned()),
                self.credentials_id.unwrap_or_else(|| DEFAULT_CREDENTIAL.to_owned())
            )?
        };

        Ok(Client {config, credentials, http_client })
    }

    /// Creates client builder from environment
    /// 
    /// The variables are supposed to have at most two optional prefixes, `env_prefix` and `env_flavor_prefix`, 
    /// with `env_prefix` leading. If there is no variable within the flavor, we fall back to no-flavor variable.
    /// 
    /// Variable base names are:
    /// 
    /// * `AK` and `SK` - access key and secret key (both required)
    /// * `SCA_CONFIG_FILE`
    /// * `SCA_CREDENTIALS_FILE`
    /// * `SCA_CREDENTIALS_ID`
    /// * `SCA_REGION`
    /// * `SCA_PROJECT_ID`
    /// 
    /// Example: if `env_prefix` == "P" and `env_flavor_prefix` == "F", the loader attempts to load the config file setting 
    /// first from `P_F_SCA_CONFIG_FILE`, then from `P_SCA_CONFIG_FILE`
    pub fn from_environment(mut self, env_prefix: Option<&str>, env_flavor_prefix: Option<&str>) -> Self {
        use std::env::var;
        let pe: Box<dyn Fn(&str) -> std::result::Result<String, _>> = match (env_prefix, env_flavor_prefix) {
            (None, None) => 
                Box::new(|s| var(s)),
            (Some(p), None) => 
                Box::new(move |s: &str| var(&format!("{p}_{s}"))),
            (None, Some(f)) => 
                Box::new(move |s: &str| var(&format!("{f}_{s}")).or_else(|_| var(s))),
            (Some(p), Some(f)) => 
                Box::new(move |s: &str| var(&format!("{p}_{f}_{s}")).or_else(|_| var(&format!("{p}_{s}")))),
        };
        
        if let (Ok(ak), Ok(sk)) = (pe("AK"), pe("SK")) {
            self = self.credentials(Credentials { ak, sk })
        }
        if let Ok(config_file) = pe("SCA_CONFIG_FILE") {
            self = self.config_file(&config_file)
        }
        if let Ok(credentials_file) = pe("SCA_CREDENTIALS_FILE") {
            self = self.credentials_file(&credentials_file)
        }
        if let Ok(credentials_id) = pe("SCA_CREDENTIALS_ID") {
            self = self.credentials_id(&credentials_id)
        }
        if let Ok(region) = pe("SCA_REGION") {
            self = self.region(&region)
        }
        if let Ok(project_id) = pe("SCA_PROJECT_ID") {
            self = self.project_id(&project_id)
        }

        self
    }

}
