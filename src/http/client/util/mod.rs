//! Utility functions for the client

/// Load account from database and map them to AccountVisitor
pub mod account_manager;

/// Get the reqwest client with the proxy and endpoint
pub mod get_reqwest_client;

/// Custom SSE processor, which can extract the truncated json from the stream
/// Q: Why we need this?
/// A: This project was originally designed to handle some additional tasks, in those tasks, the SSE stream encountered
///    is not always split according to the standard `\n\n`, so a custom processor is needed to handle this situation.
///    But later it was found that this processor can also work normally when processing standard SSE streams,
///    and the performance is also good, so it has been used all the time.
pub mod truncated_json_processor;

/// The channel manager, which is responsible for managing the buffer and sending the response to the client
pub mod counter;
