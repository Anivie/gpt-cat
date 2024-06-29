use axum::http::HeaderMap;
use log::error;
use crate::data::config::runtime_data::GlobalData;
use crate::http::client::client_sender::channel_manager::{ChannelSender, ClientSender, ResponsiveError};
use crate::http::server::ClientJoinPreHandler;

pub(super) mod title_catcher;
pub(super) mod command_handler;
pub(super) mod userid_handler;
pub(super) mod user_key_handler;
pub(super) mod model_filter;

macro_rules! impl_client_join_handler {
    ($($variant:ident),*) => {
        use crate::http::server::pre_handler::ClientJoinPreHandlerImpl;
        use crate::http::server::pre_handler::ClientJoinHandlers;
        use crate::http::server::pre_handler::ClientJoinContext;

        #[derive(Clone)]
        pub enum ClientJoinPreHandler {
            $(
                $variant($variant),
            )*
        }

        impl ClientJoinPreHandlerImpl for ClientJoinPreHandler {
            async fn client_join<'a>(&'a self, context: &mut ClientJoinContext<'a>) -> anyhow::Result<Option<String>> {
                match self {
                    $(
                        ClientJoinPreHandler::$variant(handler) => handler.client_join(context).await,
                    )*
                }
            }
        }

        pub fn get_client_join_handler() -> ClientJoinHandlers {
            ClientJoinHandlers::new(vec![
                $(
                    ClientJoinPreHandler::$variant($variant::default()),
                )*
            ])
        }
    }
}

#[allow(dead_code)]
pub struct ClientJoinContext<'a> {
    pub sender: ClientSender,
    pub user_key: Option<String>,
    pub user_id: Option<i32>,
    pub request_header: &'a HeaderMap,
    pub global_data: &'static GlobalData,
}

pub trait ClientJoinPreHandlerImpl {
    async fn client_join<'a>(&'a self, context: &mut ClientJoinContext<'a>) -> anyhow::Result<Option<String>>;
}

pub struct ClientJoinHandlers {
    handlers: Vec<ClientJoinPreHandler>
}

impl ClientJoinHandlers {
    pub fn new(inner: Vec<ClientJoinPreHandler>) -> Self {
        Self {
            handlers: inner
        }
    }

    pub async fn client_join<'a>(&'a self, mut context: ClientJoinContext<'a>) -> ClientJoinContext {
        for handler in self.handlers.iter() {
            if let Err(error) = handler.client_join(&mut context).await {
                context.sender.append_error(ResponsiveError {
                    component: "预处理器".to_string(),
                    reason: "阻止了您的会话".to_string(),
                    message: error.to_string(),
                    suggestion: None,
                });
                error!("ClientJoinHandlers: {}", error.to_string());
                break;
            };
        }

        context
    }
}