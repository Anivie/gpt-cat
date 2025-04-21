use std::error::Error;
use reqwest::StatusCode;

use crate::data::config::entity::runtime_data::AccountVisitor;
use crate::data::http_api::alibaba::qian_wen_request::{Input, Parameters, QianWenRequest};
use crate::data::http_api::alibaba::qian_wen_response::QianWenResponse;
use crate::http::client::client_sender::channel_manager::{
    ChannelBufferManager, ChannelSender, ClientSender,
};
use crate::http::client::specific_responder::{ResponderError, ResponseParser, SpecificResponder};

/// The parser for the QianWen responder
#[derive(Default)]
pub struct QianWenResponderParser;

impl ResponseParser for QianWenResponderParser {
    async fn parse_response(
        &mut self,
        sender: &mut ClientSender,
        response: &[u8],
    ) -> Result<(), ResponderError> {
        match (
            serde_json::from_slice::<QianWenResponse>(response),
            sender.request.is_stream(),
        ) {
            (Err(err), _) => {
                return Err(ResponderError::Request(format!(
                    "Error when parse response from serde: {}, origin text: {}",
                    err,
                    String::from_utf8_lossy(response)
                )));
            }

            (Ok(response), false) => {
                if let Some(choice) = response.output.choices.first() {
                    let content = &choice.message.content;
                    sender.append_buffer(content.as_str());
                }
            }

            (Ok(response), true) => {
                if let Some(choice) = response.output.choices.first() {
                    let content = &choice.message.content;
                    sender.append_buffer(content.as_str());
                    sender
                        .send_text(content, choice.finish_reason == "stop".to_string())
                        .await
                        .map_err(|e| ResponderError::Response(e.to_string()))?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct QianWenResponder;

impl SpecificResponder for QianWenResponder {
    async fn make_response(
        &self,
        sender: &mut ClientSender,
        accessor: &AccountVisitor,
    ) -> Result<(), ResponderError> {
        let stream = accessor
            .client
            .post(accessor.endpoint_url)
            .header(
                "X-DashScope-SSE",
                if sender.is_stream() {
                    "enable"
                } else {
                    "disable"
                },
            )
            .json(&QianWenRequest {
                model: sender.request.model.clone(),
                input: Input {
                    messages: sender.request.messages.clone(),
                },
                parameters: Parameters {
                    incremental_output: if sender.is_stream() { Some(true) } else { None },
                    result_format: "message".to_string(),
                },
            })
            .send()
            .await
            .map_err(|e| ResponderError::Request(format!("Error when send request: {}, reason: {:?}", e, e.source())))?;

        if stream.status() != StatusCode::OK {
            return Err(ResponderError::Request(format!(
                "Error when get response with code: {}, error message: {}",
                stream.status(),
                stream
                    .text()
                    .await
                    .map_err(|e| ResponderError::Request(e.to_string()))?
            )));
        }

        process_stream!(stream, QianWenResponderParser::default(), sender);

        Ok(())
    }
}
