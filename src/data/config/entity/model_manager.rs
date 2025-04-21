use hashbrown::{HashMap, HashSet};
use std::fs::File;
use crate::data::config::entity::config_file::Config;
use crate::data::config::entity::endpoint::Endpoint;
use crate::data::config::entity::model_mapping::ModelMapping;

type ModelInfo = HashMap<Endpoint, HashSet<String>>;

/// The manager of the model, because we need to know which model is available for each endpoint,
/// so we need to store the model info in memory.
/// # Fields
/// - global_info: The global model info, which is used to check if the model is available for any endpoints.
/// - info: The model info of each endpoint, which is used to check if the model is available for each endpoint.
pub struct ModelManager {
    global_info: HashSet<String>,
    info: ModelInfo,
}

impl ModelManager {
    pub(crate) fn new(config: &Config) -> anyhow::Result<Self> {
        let file = File::open("./config/model.json").expect("Unable to open model file.");
        let mut info = {
            let info: HashMap<String, HashSet<String>> = serde_json::from_reader(file).expect("Unable to read json");
            info
                .into_iter()
                .map(|(key, value)| {
                    let endpoint = Endpoint::from_str(key.leak(), config)?;
                    Ok((endpoint, value))
                })
                .collect::<anyhow::Result<HashMap<_, _>>>()?
        };

        let file = File::open("./config/model_mapping.json")
            .expect("Unable to open model mapping file.");
        let mapping: ModelMapping = serde_json::from_reader(file).expect("Unable to read json");

        for (endpoint, value) in mapping.iter() {
            if let Some(set) = info.get_mut(endpoint) {
                for (_, after) in value.iter() {
                    set.insert(after.to_string());
                }
            }
        }

        let mut global = HashSet::new();
        for (_, value) in info.iter() {
            for model in value.iter() {
                global.insert(model.to_string());
            }
        }

        Ok(ModelManager {
            info,
            global_info: global,
        })
    }
}

impl ModelManager {
    /// Check if the model is available for the endpoint.
    pub fn check_available(&self, endpoint: &Endpoint, model: &str) -> bool {
        self
            .info
            .get(endpoint)
            .map(|x| x.contains(model))
            .unwrap_or(false)
    }

    /// Check if the model is available for any endpoint.
    pub fn has_model(&self, model: &str) -> bool {
        self.global_info.contains(model)
    }
}
