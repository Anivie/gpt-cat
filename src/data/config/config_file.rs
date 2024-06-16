use std::ops::Deref;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::data::config::endpoint::Endpoint;

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
    pub database_url: String,

    pub number_can_retries: u32,
    pub request_concurrency_count: u32,

    pub proxy: Option<Vec<ProxyConfig>>,
}

/// The proxy config that can be used in each endpoint key.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub scheme: String,
    pub address: String,
    pub name: String,
    pub password: String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMap(DashMap<Endpoint, String>);

impl PartialEq for EndpointMap {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0.clone().into_iter().all(|(key, value)| {
            other.0.get(&key).map_or(false, |v| v.deref() == &value)
        })
    }
}

impl Deref for EndpointMap {
    type Target = DashMap<Endpoint, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}