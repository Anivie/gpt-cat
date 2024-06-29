//! Utility functions for the client

/// Load account from database and map them to AccountVisitor
pub mod account_manager;

/// Get the reqwest client with the proxy and endpoint
pub mod get_reqwest_client;

/// The channel manager, which is responsible for managing the buffer and sending the response to the client
pub mod counter;
pub mod sse;
