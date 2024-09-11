use log::info;
use ntex::util::Bytes;
use ntex::web;
use ntex::web::types::{Json, State};
use ntex::web::{HttpRequest, Responder};
use std::ops::Deref;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc::channel;

use crate::data::config::entity::runtime_data::ServerPipeline;
use crate::data::openai_api::openai_request::OpenAIRequest;
use crate::http::client::client_sender::channel_manager::{ChannelSender, ClientSender};
use crate::http::server::after_handler::ClientEndContext;
use crate::http::server::pre_handler::ClientJoinContext;
use crate::http::server::web::enum_response::end;
use crate::GlobalData;

/// The main chat handler
/// This handler will handle the main chat request
/// The main chat request will be handled by the pipeline,
/// which will handle the request in the pre_handler, if
/// the request is valid, we will choose an endpoint from
/// account pool, then send the request to the endpoint and
/// parse it in OpenAI format, then send the response to the
/// client.
/// The pipeline will handle the request in the after_handler
/// after the request is done.
/// # Note that
///  - The pre-handler pipeline running in the sync mode, which means
///    that any of them should wait for the previous one to finish, and
///    any can modify the request data.
///    **Any Error response by them will stop the request, so you can
///    regard them as a request filter**
/// -  The after-handler pipeline running in the async mode, which means
///    that any of them can run concurrently, but they can't modify the
///    request data and can't stop the request.
/// # Parameters
/// - headers: The request headers
/// - data: The global data
/// - pipeline: The server pipeline
/// - client_request: The client request by the user
/// # Returns
/// The response data either an SSE or a json, depends on the request is a
/// stream or not
#[web::post("/v1/chat/completions")]
pub async fn main_chat(
    request: HttpRequest,
    state: State<(&'static GlobalData, &'static ServerPipeline)>,
    client_request: Json<OpenAIRequest>,
) -> impl Responder {
    let &(data, pipeline) = state.deref();
    let (sender, receiver) = channel::<Bytes>(10);
    let sender = ClientSender::new(sender, client_request.into_inner());

    let pre_handler_context = ClientJoinContext {
        sender,
        user_key: None,
        user_id: None,
        request_header: &request.head().headers,
        global_data: data,
    };

    let client_request = pipeline.pre_handler.client_join(pre_handler_context).await;
    if client_request.sender.stopped {
        client_request.sender.send_error().await.unwrap();
        return end(receiver, client_request.sender.is_stream()).await;
    }

    let user_id = client_request.user_id.clone().unwrap();
    let mut sender = client_request.sender;
    let is_stream = sender.request.stream.unwrap_or(false);

    info!("User {} start request......", user_id);

    spawn(async move {
        if let Some(response_data) = data.try_request(&mut sender).await {
            let after_context = ClientEndContext {
                sender,
                response_data,
                user_id,
                data,
            };

            let after_context = Arc::new(after_context);
            let handlers_result = pipeline
                .after_handler
                .client_end(after_context)
                .await
                .expect("Error when start after handler");

            for handler_future in handlers_result {
                handler_future
                    .await
                    .expect("Error when run after handler")
                    .expect("Error when try after handler");
            }
        }

        info!("End of the request: Done.");
    });

    end(receiver, is_stream).await
}