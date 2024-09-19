use crate::http::server::pre_handler::command::handlers::say_hi::SayHi;
use crate::http::server::pre_handler::command::handlers::template::TemplateHandler;

pub mod command_handler;
#[macro_use]
mod handlers;

command_handler_dispatcher! [
    SayHi,
    TemplateHandler
];