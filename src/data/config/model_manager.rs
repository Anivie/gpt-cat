use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;

use dashmap::DashMap;
use log::info;
use crate::data::config::endpoint::Endpoint;


type ModelInfo = DashMap<Endpoint, HashSet<String>>;

/// The manager of the model, because we need to know which model is available for each endpoint,
/// so we need to store the model info in memory.
/// # Fields
/// - global_info: The global model info, which is used to check if the model is available for any endpoints.
/// - info: The model info of each endpoint, which is used to check if the model is available for each endpoint.
pub struct ModelManager {
    global_info: HashSet<String>,
    info: ModelInfo,
}

impl Default for ModelManager {
    fn default() -> Self {
        let file = File::open("./config/model.json").expect("Unable to open model file.");
        let config = BufReader::new(file);
        let config: ModelInfo = serde_json::from_reader(config).expect("Unable to read json");

        let mut global = HashSet::new();
        for value in config.iter() {
            for model in value.value().iter() {
                global.insert(model.to_string());
            }
        }

        ModelManager {
            global_info: global,
            info: config
        }
    }
}

impl ModelManager {
    /// Check if the model is available for the endpoint.
    pub fn check_available(&self, endpoint: &Endpoint, model: &str) -> bool {
        let back = self.info.get(endpoint).unwrap().contains(model);
        if back {
            info!("Use of model: {}", model);
        }
        back
    }

    /// Check if the model is available for any endpoint.
    pub fn has_model(&self, model: &str) -> bool {
        self.global_info.contains(model)
    }
}