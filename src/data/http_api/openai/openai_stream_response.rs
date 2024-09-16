use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenAIStreamResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: u64,
    pub model: Option<String>,
    pub system_fingerprint: Option<String>,
    pub choices: Vec<Choice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Choice {
    pub index: i64,
    pub delta: Delta,
    pub logprobs: Option<String>,
    pub finish_reason: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

impl OpenAIStreamResponse {
    pub fn new(model_name: String, answer: &str, end: bool) -> OpenAIStreamResponse {
        OpenAIStreamResponse {
            id: Some("chatcmpl-123".to_string()),
            object: Some("chat.completion.chunk".to_string()),
            created: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            model: Some(model_name),
            system_fingerprint: Some("fp_44709d6fcb".to_string()),
            choices: vec![Choice {
                index: 0,
                delta: Delta {
                    role: Some("assistant".to_string()),
                    content: Some(answer.to_string()),
                },
                logprobs: None,
                finish_reason: if end { Some("stop".to_string()) } else { None },
            }],
        }
    }
}
