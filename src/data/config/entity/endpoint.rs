use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Deref;
use anyhow::bail;
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use crate::data::config::entity::config_file::Config;

/// Supported endpoint of this server, default have OpenAI and QianWen endpoint.
/// This app is fully type safe, so you can add a new endpoint here,
/// and then rustc will tell you what you need to do.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, EnumIter)]
pub enum Endpoint {
    OpenAI,
    QianWen,
    Alias(Cow<'static, str>, Box<Endpoint>),
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endpoint::OpenAI => write!(f, "OpenAI"),
            Endpoint::QianWen => write!(f, "QianWen"),
            Endpoint::Alias(name, _) => write!(f, "{}", name),
        }
    }
}

impl Endpoint {
    /// Get the url of this endpoint.
    /// This function will return the url of this endpoint, if the url is not found in config,
    /// it will return the default url of this endpoint.
    pub fn to_url(&self, config: &Config) -> Result<String, anyhow::Error> {
        config
            .endpoint
            .get(self)
            .map(|url| url.as_str())
            .or(self.default_url().ok())
            .or_else(|| {
                match self {
                    Endpoint::Alias(from, to) => {
                        config.endpoint_mapping
                            .as_ref()
                            .and_then(|mapping| mapping.get(from.deref()))
                            .map(|x| x.1.as_deref())
                            .unwrap_or_else(|| { to.default_url().ok() })
                    }
                    _ => None,
                }
            })
            .map(|url| url.to_string())
            .ok_or_else(|| anyhow::anyhow!("Endpoint {} not found in config", self))
    }

    /// Get the endpoint from string. All the strings are all caps.
    /// This function will return the endpoint from string if the string is not found in config,
    /// it will panic
    /// **IMPORTANT NOTE**
    /// Because of string cannot be enumeration, this function will not return a result, but will panic if
    /// you didn't pass a valid name for every endpoint.
    pub fn from_str(s: &'static str, config: &Config) -> anyhow::Result<Endpoint> {
        let endpoint = match s {
            "OpenAI" => Some(Endpoint::OpenAI),
            "QianWen" => Some(Endpoint::QianWen),
            _ => None,
        };

        endpoint
            .or_else(|| {
                config
                    .endpoint_mapping
                    .as_ref()
                    .and_then(|x| { x.get(s) })
                    .cloned()
                    .map(|x| Endpoint::Alias(Cow::from(s), Box::new(x.0)))
            })
            .ok_or(anyhow::anyhow!("Endpoint {} not found in config", s))
    }

    /// Get the default url of this endpoint.
    /// This will be used when the url is not found in config.
    fn default_url(&self) -> anyhow::Result<&str> {
        match self {
            Endpoint::OpenAI => Ok("https://api.openai.com/v1/chat/completions"),
            Endpoint::QianWen => Ok("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation"),
            Endpoint::Alias(_,_) => bail!("Unknown url for endpoint: {}", self),
        }
    }
}


impl Default for Endpoint {
    fn default() -> Self {
        Endpoint::OpenAI
    }
}