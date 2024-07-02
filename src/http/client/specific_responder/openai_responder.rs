use reqwest::StatusCode;

use crate::data::config::runtime_data::AccountVisitor;
use crate::data::openai_api::openai_sync_response::OpenAISyncResponse;
use crate::data::openai_api::openai_stream_response::OpenAIStreamResponse;
use crate::http::client::client_sender::channel_manager::{ChannelBufferManager, ChannelSender, ClientSender};
use crate::http::client::specific_responder::{ResponderError, ResponseParser, SpecificResponder};

#[derive(Default)]
pub struct OpenAIResponder;

/// The parser for the OpenAI response
#[derive(Default)]
struct OpenAIResponderParser;

impl ResponseParser for OpenAIResponderParser {
    async fn parse_response(&mut self,
                            sender: &mut ClientSender,
                            response: &[u8]
    ) -> Result<(), ResponderError> {
        if sender.request.is_stream() {
            match serde_json::from_slice::<OpenAIStreamResponse>(response) {
                Err(err) => {
                    return Err(ResponderError::Request(format!("Error when parse response from serde: {}, origin text: {}", err, String::from_utf8_lossy(response))));
                }

                Ok(response) => {
                    if let Some(choice) = response.choices.first() &&
                        let Some(content) = &choice.delta.content
                    {
                        sender.append_buffer(content.as_str());
                        sender.send_text(
                            content,
                            response.choices[0].finish_reason == Some("stop".to_string())
                        ).await.map_err(|e| ResponderError::Response(e.to_string()))?;
                    }
                }
            }

            return Ok(());
        }

        match serde_json::from_slice::<OpenAISyncResponse>(response) {
            Err(err) => {
                return Err(ResponderError::Request(format!("Error when parse response from serde: {}, origin text: {}", err, String::from_utf8_lossy(response))));
            }

            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    sender.append_buffer(choice.message.content.as_str());
                }
            }
        }

        Ok(())
    }
}

impl SpecificResponder for OpenAIResponder {
    async fn make_response(&self,
                           sender: &mut ClientSender,
                           accessor: &AccountVisitor
    ) -> Result<(), ResponderError> {
        let stream = accessor
            .client
            .post(accessor.endpoint_url.clone())
            .body(serde_json::to_string(&sender.request).map_err(|e| ResponderError::Request(e.to_string()))?)
            .send()
            .await
            .map_err(|e| ResponderError::Request(format!("Error when send request: {}", e)))?;

        if stream.status() != StatusCode::OK {
            return Err(ResponderError::Request(format!(
                "Error when get response with code: {}, error message: {}",
                stream.status(),
                stream.text().await.map_err(|e| ResponderError::Request(e.to_string()))?
            )));
        }

        process_stream!(stream, OpenAIResponderParser::default(), sender);

        Ok(())
    }
}