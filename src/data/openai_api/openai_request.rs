use std::fmt::{Display, Formatter};
use std::ops::Deref;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OpenAIRequest is a struct that represents the request that will be sent to the OpenAI API.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenAIRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
    pub presence_penalty: f32,
    pub frequency_penalty: f32,
    pub top_p: f32,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Value>,
}

impl OpenAIRequest {
    #[inline]
    pub fn is_stream(&self) -> bool {
        self.stream.unwrap_or(false)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Common(String),
    File(Vec<FileMessageContent>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FileMessageContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    url: String,
}

impl Default for MessageContent {
    fn default() -> Self {
        MessageContent::Common(String::default())
    }
}

impl Deref for MessageContent {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            MessageContent::Common(content) => content,
            MessageContent::File(f) => match f.first() {
                None => {
                    panic!("No file content")
                }
                Some(x) => match x {
                    FileMessageContent::Text { text } => text,
                    FileMessageContent::ImageUrl { image_url } => &image_url.url,
                },
            },
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Message: [role: {}, content: {:?}]",
            self.role, self.content
        )
    }
}

#[allow(dead_code)]
pub enum MessageLocation {
    FIRST,
    LAST,
}

pub trait MessageUtil {
    fn get_user_input(&self, location: MessageLocation) -> Option<&str>;
    fn get_all_input(&self) -> InputMessageContent<'_>;
}

pub struct InputMessageContent<'a> {
    inner: &'a Vec<Message>,
    slice: Vec<&'a str>,
}

impl Display for InputMessageContent<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut back = String::new();
        for message in self.inner {
            back.push('[');
            back.push_str(message.role.deref());
            back.push(']');
            back.push(' ');
            back.push_str(message.content.deref());
            back.push('\n');
        }

        write!(f, "{}", back)
    }
}

impl<'a> Deref for InputMessageContent<'a> {
    type Target = Vec<&'a str>;

    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl MessageUtil for Vec<Message> {
    #[inline]
    fn get_user_input(&self, location: MessageLocation) -> Option<&str> {
        let option = self
            .par_iter()
            .filter(|x| x.role == "user")
            .collect::<Vec<_>>();

        let option = match location {
            MessageLocation::FIRST => option.first(),
            MessageLocation::LAST => option.last(),
        };

        match option {
            None => None,
            Some(&content) => Some(content.content.deref()),
        }
    }

    fn get_all_input(&self) -> InputMessageContent<'_> {
        InputMessageContent {
            inner: self,
            slice: self
                .par_iter()
                .map(|x| x.content.deref())
                .collect::<Vec<_>>(),
        }
    }
}

#[allow(dead_code)]
pub trait FancyWaysObtainLength {
    fn get_user_length(&self) -> usize;
    fn get_user_and_assistant_length(&self) -> usize;
    fn get_system_length(&self) -> usize;
}

impl FancyWaysObtainLength for Vec<Message> {
    #[inline]
    fn get_user_length(&self) -> usize {
        self.par_iter().filter(|x| x.role == "user").count()
    }

    #[inline]
    fn get_user_and_assistant_length(&self) -> usize {
        self.par_iter()
            .filter(|x| x.role == "user" || x.role == "assistant")
            .count()
    }

    #[inline]
    fn get_system_length(&self) -> usize {
        self.par_iter().filter(|x| x.role == "system").count()
    }
}

#[test]
fn test_openai_requests() {
    let json = r#"{
	"model": "gpt-3.5-turbo",
	"stream": true,
	"frequency_penalty": 0,
	"presence_penalty": 0,
	"temperature": 0.6,
	"top_p": 1,
	"messages": [{
		"content": "Tools\n\nYou can use these tools below:\n\n### Search Google via Serper\n\nPlugin for performing web searches using the Serper.dev API to access Google search results.\n\nThe APIs you can use:\n\n#### search-engine-serper____searchGoogle\n\nSearch Google and return top 10 results",
		"role": "system"
	}, {
		"content": "搜索最新的政治新闻",
		"role": "user"
	}],
	"tools": [{
		"function": {
			"description": "Search Google and return top 10 results",
			"name": "search-engine-serper____searchGoogle",
			"parameters": {
				"properties": {
					"q": {
						"type": "string"
					},
					"gl": {
						"type": "string"
					},
					"hl": {
						"type": "string"
					}
				},
				"required": ["q"],
				"type": "object"
			}
		},
		"type": "function"
	}]
}"#;

    serde_json::from_str::<OpenAIRequest>(json).unwrap();
}
