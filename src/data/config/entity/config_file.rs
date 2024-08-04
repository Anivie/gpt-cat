use std::ops::Deref;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::data::config::entity::endpoint::Endpoint;

const fn default_number_can_retries() -> u32 { 3 }
const fn default_request_concurrency_count() -> u32 { 10 }
fn default_address() -> String { "0.0.0.0".to_string() }
const fn default_http_address() -> u16 { 7117 }
const fn default_https_address() -> u16 { 11711 }

/// The config file of the server.
/// This will be read from ./config/config.json
/// # Fields
/// - endpoint: A map of each endpoint, save the url for the endpoint.
/// - database_url: The database url of the server.
/// - number_can_retries: The number of retries when the request fails.
/// - request_concurrency_count: The number of concurrent requests.
/// - proxy: The proxy server use if an account specified.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub endpoint: EndpointMap,

    #[serde(default)]
    pub database_url: String,

    #[serde(default = "default_number_can_retries")]
    pub number_can_retries: u32,
    #[serde(default = "default_request_concurrency_count")]
    pub request_concurrency_count: u32,

    #[serde(flatten)]
    pub http_config: HttpServerConfig,

    pub proxy: Option<Vec<ProxyConfig>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpServerConfig {
    #[serde(default)]
    pub enable_https: bool,

    #[serde(default = "default_address")]
    pub http_address: String,
    #[serde(default = "default_http_address")]
    pub http_port: u16,

    #[serde(default = "default_address")]
    pub https_address: String,
    #[serde(default = "default_https_address")]
    pub https_port: u16,
}

/// The proxy config that can be used in each endpoint key.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub scheme: String,
    pub address: String,
    pub name: String,
    pub password: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMap(DashMap<Endpoint, String>);

impl PartialEq for EndpointMap {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0
            .clone()
            .into_iter()
            .all(|(key, value)| other.0.get(&key).map_or(false, |v| v.deref() == &value))
    }
}

impl Deref for EndpointMap {
    type Target = DashMap<Endpoint, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
