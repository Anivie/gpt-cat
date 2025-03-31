use crate::data::config::entity::endpoint::Endpoint;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// A map contains the price of the model for user to use.
/// # Fields
/// - inner: The inner map, which contains the price of the model.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelMapping {
    #[serde(flatten)]
    inner: HashMap<Endpoint, HashMap<String, String>>,
}

impl Deref for ModelMapping {
    type Target = HashMap<Endpoint, HashMap<String, String>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for ModelMapping {
    fn default() -> Self {
        let file = std::fs::File::open("./config/model_mapping.json")
            .expect("Unable to open model mapping file.");
        serde_json::from_reader(file).expect("Unable to read json")
    }
}