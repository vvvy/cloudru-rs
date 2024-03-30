use super::config::*;
use crate::*;

pub(crate) type ServiceId = &'static str;

pub struct Client<HC> {
    pub(crate) config: Config,
    pub(crate) aksk: AkSk,
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
            Err(crate::error::CloudRuInnerError::MissingProjectId.into())
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
    pub aksk: Option<AkSk>,
}

impl ClientBuilder {
    pub fn new() -> Self { Self::default() }
    pub fn config_file(self, arg: &str) -> Self { Self { config_file: Some(arg.to_owned()), ..self } }
    pub fn credentials_file(self, arg: &str) -> Self { Self { credentials_file: Some(arg.to_owned()), ..self } }
    pub fn credentials_id(self, arg: &str) -> Self { Self { credentials_id: Some(arg.to_owned()), ..self } }
    pub fn project_id(self, arg: &str) -> Self { Self { project_id: Some(arg.to_owned()), ..self } }
    pub fn region(self, arg: &str) -> Self { Self { region: Some(arg.to_owned()), ..self } }
    pub fn aksk(self, arg: AkSk) -> Self { Self { aksk: Some(arg), ..self } }
    pub fn build_with_http_client<HC>(self, http_client: HC) -> Result<Client<HC>> {
        let (config_path, force) = self.config_file
            .map(|f| (f, true))
            .unwrap_or_else(|| (DEFAULT_CONFIG_FILE.to_owned(), false));

        let mut config = read_config(config_path, force)?;
        //config.project_id = self.project_id.or(config.project_id);
        if let Some(project_id) = self.project_id { config.project_id = Some(project_id); }
        if let Some(region) = self.region { config.region = region; }
        
        let aksk = if let Some(aksk) = self.aksk {
            aksk
        } else {
            read_credentials(
                self.credentials_file.unwrap_or_else(|| DEFAULT_CREDENTIALS_FILE.to_owned()),
                self.credentials_id.unwrap_or_else(|| DEFAULT_CREDENTIAL.to_owned())
            )?
        };

        Ok(Client {config, aksk, http_client })
    }
}