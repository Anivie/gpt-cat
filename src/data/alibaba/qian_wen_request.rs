use serde::{Deserialize, Serialize};

use crate::data::openai_api::openai_request::Message;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianWenRequest {
    pub model: String,
    pub input: Input,
    pub parameters: Parameters,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub messages: Vec<Message>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub incremental_output: Option<bool>,
    pub result_format: String,
}
