//! This module contains the main server logic and the enum for the response.
//! This app uses the axum framework to handle the http request and response.

use std::sync::Arc;

use crate::http::server::after_handler::token_meter::TokenMeterHandler;
use crate::http::server::pre_handler::command_handler::CommandHandler;
use crate::http::server::pre_handler::model_filter::ModelFilterHandler;
use crate::http::server::pre_handler::title_catcher::TitleCatchHandler;
use crate::http::server::pre_handler::user_key_handler::UserKeyHandler;
use crate::http::server::pre_handler::userid_handler::UserIDHandler;

/// The pre-handler pipeline
#[macro_use]
pub mod pre_handler;

/// The after-handler pipeline
#[macro_use]
pub mod after_handler;

/// The web server
pub mod web;

/// Define the pre-handler pipeline, this pipeline running when a
/// client request is coming. **Note that** the order of the handler
/// is important, the handler will be executed in the order of the
/// definition, and the input of the next handler is the output of the
/// previous handler.
impl_client_join_handler![
    ModelFilterHandler,
    UserKeyHandler,
    UserIDHandler,
    TitleCatchHandler,
    CommandHandler
];

/// Define the after-handler pipeline, because the after-handler is
/// running in the async mode, so the order of the handler is not
/// important
impl_client_end_handler![TokenMeterHandler];
