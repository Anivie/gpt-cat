use std::fs::File;
use crate::data::config::entity::endpoint::Endpoint;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use crate::data::config::entity::config_file::Config;

/// A map contains the price of the model for user to use.
/// # Fields
/// - inner: The inner map, which contains the price of the model.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelMapping {
    #[serde(flatten)]
    pub inner: HashMap<Endpoint, HashMap<String, String>>,
}

impl Deref for ModelMapping {
    type Target = HashMap<Endpoint, HashMap<String, String>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ModelMapping {
    pub(crate) fn new(config: &Config) -> anyhow::Result<Self> {
        let file = File::open("./config/model_mapping.json")
            .expect("Unable to open model mapping file.");
        
        let mapping: HashMap<String, HashMap<String, String>> = serde_json::from_reader(file).expect("Unable to read json");
        let mapping = mapping
            .into_iter()
            .map(|(key, value)| {
                let endpoint = Endpoint::from_str(key.leak(), config)?;
                Ok((endpoint, value))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        Ok(ModelMapping {
            inner: mapping,
        })
    }
}