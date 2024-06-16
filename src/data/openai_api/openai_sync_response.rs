use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenAISyncResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: u64,
    pub model: Option<String>,
    pub system_fingerprint: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub index: u8,
    pub message: Message,
    pub logprobs: Option<String>,
    pub finish_reason: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

impl OpenAISyncResponse {
    pub fn new(model_name: String, answer: &str, end: bool) -> OpenAISyncResponse {
        OpenAISyncResponse {
            id: Some("chatcmpl-123".to_string()),
            object: Some("chat.completion".to_string()),
            created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            model: Some(model_name),
            system_fingerprint: Some("fp_44709d6fcb".to_string()),
            choices: vec![
                crate::data::openai_api::openai_sync_response::Choice {
                    index:0,
                    logprobs:None,
                    finish_reason:if end {
                        Some("stop".to_string())
                    }else {
                        None
                    },
                    message: crate::data::openai_api::openai_sync_response::Message {
                        role: "assistant".to_string(),
                        content: answer.to_string()
                    }
                }
            ],
            usage: Default::default(),
        }
    }
}