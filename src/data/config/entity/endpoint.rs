use std::fmt::{Display, Formatter};

use crate::data::config::entity::config_file::Config;
use log::warn;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// Supported endpoint of this server, default have OpenAI and QianWen endpoint.
/// This app is fully type safe, so you can add a new endpoint here,
/// and then rustc will tell you what you need to do.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, EnumIter)]
pub enum Endpoint {
    OpenAI,
    QianWen,
}

impl Endpoint {
    /// Get the url of this endpoint.
    /// This function will return the url of this endpoint, if the url is not found in config,
    /// it will return the default url of this endpoint.
    pub fn to_url(&self, config: &Config) -> String {
        let back = config.endpoint.get(self);
        match back {
            Some(back) => back.clone(),
            None => {
                warn!("Endpoint {} not found in config, use default url.", self);
                self.default_url().to_string()
            }
        }
    }

    /// Get the endpoint from string. All the strings are all caps.
    /// This function will return the endpoint from string if the string is not found in config,
    /// it will panic
    /// **IMPORTANT NOTE**
    /// Because of string cannot be enumeration, this function will not return a result, but will panic if
    /// you didn't pass a valid name for every endpoint.
    pub fn from_str(s: &str) -> Endpoint {
        match s.to_ascii_uppercase().as_str() {
            "OPENAI" => Endpoint::OpenAI,
            "QIANWEN" => Endpoint::QianWen,
            _ => panic!("Unknown endpoint: {}", s),
        }
    }

    /// Get the default url of this endpoint.
    /// This will be used when the url is not found in config.
    const fn default_url(&self) -> &str {
        match self {
            Endpoint::OpenAI => "https://api.openai.com/v1/chat/completions",
            Endpoint::QianWen => {
                "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation"
            }
        }
    }
}

/// Display the endpoint name.
impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Endpoint::OpenAI => write!(f, "OpenAI"),
            Endpoint::QianWen => write!(f, "QianWen"),
        }
    }
}
