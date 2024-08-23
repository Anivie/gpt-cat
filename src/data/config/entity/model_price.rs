use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use hashbrown::HashMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

type ModelName = String;

/// A map contains the price of the model for user to use.
/// # Fields
/// - inner: The inner map, which contains the price of the model.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelPriceMap {
    #[serde(flatten)]
    inner: HashMap<ModelName, ModelPriceValue>,
}

/// The value of the model price.
/// # Fields
/// - input_price: The price of the input.
/// - output_price: The price of the output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelPriceValue {
    pub input_price: Decimal,
    pub output_price: Decimal,
}

impl Deref for ModelPriceMap {
    type Target = HashMap<ModelName, ModelPriceValue>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for ModelPriceMap {
    fn default() -> Self {
        let file = File::open("./config/model_price.json").expect("Unable to open model file.");
        let config = BufReader::new(file);

        serde_json::from_reader(config).expect("Unable to read json")
    }
}
