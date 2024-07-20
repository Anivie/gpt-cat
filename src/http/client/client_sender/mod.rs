//! This module contains the main logic for the channel manager.
//! The channel manager is responsible for managing the channels that are
//! used to communicate with the client.
//! It is responsible for creating new channels, closing channels, and
//! sending messages to the client.
//! The channel manager is also responsible for handling incoming messages
//! from the client and routing them to the appropriate channel.
//! The channel manager is implemented as a singleton for each request.
pub mod channel_manager;
