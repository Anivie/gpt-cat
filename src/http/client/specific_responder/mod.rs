//! This module is the responder module, which is responsible for sending the request to the endpoint and
//! parsing the response from the endpoint.
//! # Specific Responder
//! The specific responder is the trait that defines the method to make the response to the client.
//! If you want to add a new responder, just implement the SpecificResponder trait, after make
//! the request to the endpoint and return it, server will auto parse the response in the OpenAI
//! format and send it to the client.

use thiserror::Error;

use crate::data::config::runtime_data::AccountVisitor;
use crate::http::client::client_sender::channel_manager::{ChannelBufferManager, ClientSender};

#[macro_use]
mod macros;
pub mod openai_responder;
pub mod qianwen_responder;

/// The error type for the responder module
/// # Variants
/// * `Request` - Error when try to send request to endpoint
/// * `Response` - Error when try to response to client
#[derive(Error, Debug)]
pub(crate) enum ResponderError {
    #[error("Error when try to send request to endpoint: {0}")]
    Request(String),
    #[error("Error when try to response to client : {0}")]
    Response(String),
}

/// The trait that defines the method to make the response to the client.
pub trait SpecificResponder {
    async fn make_response(
        &self,
        sender: &mut ClientSender,
        accessor: &AccountVisitor,
    ) -> Result<(), ResponderError>;
}

/// The trait that defines the method to parse the response from the endpoint.
/// If you want to add a new parser, just implement the ResponseParser trait, after make
/// the request to the endpoint, use this parser or parse the response manually
/// **This trait should be used with the `process_stream!` macro**
trait ResponseParser {
    async fn parse_response(
        &mut self,
        sender: &mut ClientSender,
        response: &[u8],
    ) -> Result<(), ResponderError>;

    async fn parse_end(&mut self, sender: &ClientSender) -> Result<(), ResponderError> {
        if sender.request.is_stream() {
            return Ok(());
        }

        sender
            .push_buffer()
            .await
            .map_err(|e| ResponderError::Response(e.to_string()))?;

        Ok(())
    }
}
