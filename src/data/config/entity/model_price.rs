use hashbrown::HashMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::ops::Deref;
use crate::data::config::entity::model_mapping::ModelMapping;

type ModelName = String;

/// A map contains the price of the model for user to use.
/// # Fields
/// - inner: The inner map, which contains the price of the model.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelPriceMap {
    #[serde(flatten)]
    inner: HashMap<ModelName, ModelPriceValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelPriceValue {
    PerToken(ModelPerToken),
    PerTimes(ModelPerTimes),
}

/// The value of the model price.
/// # Fields
/// - input_price: The price of the input.
/// - output_price: The price of the output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPerToken {
    pub input_price: Decimal,
    pub output_price: Decimal,
}

/// The value of the model price.
/// # Fields
/// - price: The price of the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPerTimes {
    pub price: Decimal,
}

impl Deref for ModelPriceMap {
    type Target = HashMap<ModelName, ModelPriceValue>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for ModelPriceMap {
    fn default() -> Self {
        let file = File::open("./config/model_price.json")
            .expect("Unable to open model file.");
        let mut inner: HashMap<ModelName, ModelPriceValue> = serde_json::from_reader(file).expect("Unable to read json");

        let file = File::open("./config/model_mapping.json")
            .expect("Unable to open model mapping file.");
        let mapping: ModelMapping = serde_json::from_reader(file).expect("Unable to read json");

        for (_, value) in mapping.iter() {
            for (before, after) in value.iter() {
                if let Some(price) = inner.get(before) {
                    inner.insert(after.to_string(), price.clone());
                }
            }
        }

        Self {
            inner
        }
    }
}
