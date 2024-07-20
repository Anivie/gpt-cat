use crate::http::client::specific_responder::openai_responder::*;
use crate::http::client::specific_responder::qianwen_responder::QianWenResponder;

pub mod client;
#[macro_use]
pub mod specific_responder;
pub mod client_sender;
pub mod util;

/// Register all the specific responder in this, this macro will generate
/// the ResponderDispatcher to static dispatch the responder
/// **Note that**: Any endpoint should have a responder
impl_specific_responder![
    Endpoint::QianWen with QianWenResponder,
    Endpoint::OpenAI with OpenAIResponder
];
