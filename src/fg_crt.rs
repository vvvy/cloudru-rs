//!Function Graph Custom Runtime



use std::time::Duration;
pub use serde_json::Value;
use serde_json::json;
use tracing::{debug, info, error};
use crate::*;

pub enum FnResult {
    Success(Value),
    Error(Value)
}

pub struct FnData {
    pub function_package: String,
    pub function_name: String,
    pub function_version: String,
    pub userdata: String,
    pub coderoot: String,
}

pub trait FgFn {
    fn invoke(&mut self, value: Value) -> FnResult;
}

pub struct StatusFn {
    status: String
}
impl StatusFn {
    pub fn new(status: String) -> Self { Self { status } }
}
impl FgFn for StatusFn {
    fn invoke(&mut self, value: Value) -> FnResult {
        FnResult::Success(json!({"status": &self.status, "input": value}))
    }
}

fn readvar(n: &str) -> String { std::env::var(n).unwrap_or_else(|_| "".to_owned()) }

pub fn is_fg_env() -> bool {
    !readvar("RUNTIME_API_ADDR").is_empty()
}

/// provides bootstrap and fg handler wrapper code for json functions 
pub fn service_function<E: From<HCInnerError>+From<reqwest::Error>>(
    selector: impl FnOnce(FnData) -> std::result::Result<Box<dyn FgFn>, E>
) -> std::result::Result<(), E> {
    let api_addr = readvar("RUNTIME_API_ADDR");

    let fn_data = FnData {
        function_package: readvar("RUNTIME_PACKAGE"),
        function_name: readvar("RUNTIME_FUNC_NAME"),
        function_version: readvar("RUNTIME_FUNC_VERSION"),
        userdata: readvar("RUNTIME_USERDATA"),
        coderoot: readvar("RUNTIME_CODE_ROOT"),
    };

    if api_addr.is_empty() {
        return Err(HCInnerError::EmptyRuntimeAddr.into());
    }
    let mut lambda = selector(fn_data)?;
    let next_invocation_url = format!("http://{api_addr}/v1/runtime/invocation/request");
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(5))
        .build()?;

    let mut request_loop = || -> crate::Result<()> {

        //get next invovcation
        let request = client.get(next_invocation_url.clone()).build()?;
        let response = client.execute(request)?;
        //extract request id
        let request_id = response
            .headers()
            .get("X-Cff-Request-Id")
            .ok_or(HCInnerError::RequestIdNotFound)?
            .to_str()?
            .to_owned();

        let request_data = response.json::<Value>()?;
        info!("Incoming request, id={request_id}");
        debug!("Request data: {request_data}");

        match lambda.invoke(request_data) {
            FnResult::Success(response_data) => {
                info!("Response: success");
                debug!("Response data: {response_data}");
                let invocation_response_url = 
                    format!("http://{api_addr}/v1/runtime/invocation/response/{request_id}");
                let request = client.post(invocation_response_url).json(&response_data).build()?;
                let _ = client.execute(request)?;
            }
            FnResult::Error(response_data) => {
                info!("Response: error");
                debug!("Response data: {response_data}");
                let invocation_response_url = 
                    format!("http://{api_addr}/v1/runtime/invocation/error/{request_id}");
                let request = client.post(invocation_response_url).json(&response_data).build()?;
                let _ = client.execute(request)?;
            }            
        }
        Ok(())
    };

    loop {
        if let Err(e) = request_loop() {
            match e.inner_ref() {
                HCInnerError::Reqwest(e) if e.is_timeout() => (),
                _ => error!("Error: {:?}", e)
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}