use std::sync::Arc;
use tokio::spawn;
use tokio::task::JoinHandle;

use crate::data::config::runtime_data::GlobalData;
use crate::http::client::client::ResponseData;
use crate::http::client::client_sender::channel_manager::ClientSender;
use crate::http::server::ClientEndAfterHandle;

pub(super) mod token_meter;

macro_rules! impl_client_end_handler {
    ($($variant:ident),*) => {
        use crate::http::server::after_handler::ClientEndAfterHandlerImpl;
        use crate::http::server::after_handler::ClientEndHandlers;
        use crate::http::server::after_handler::ClientEndContext;

        #[derive(Clone)]
        pub enum ClientEndAfterHandle {
            $(
                $variant($variant),
            )*
        }

        impl ClientEndAfterHandlerImpl for ClientEndAfterHandle {
            async fn client_end(&self, context: Arc<ClientEndContext>) -> Result<(), String> {
                match self {
                    $(
                        ClientEndAfterHandle::$variant(handler) => handler.client_end(context).await,
                    )*
                }
            }
        }

        pub fn get_client_end_handler() -> ClientEndHandlers {
            ClientEndHandlers::new(vec![
               $(
                   ClientEndAfterHandle::$variant($variant::default()),
               )*
            ])
        }
    }
}

#[allow(dead_code)]
pub struct ClientEndContext {
    pub sender: ClientSender,
    pub response_data: ResponseData,
    pub user_id: i32,
    pub data: &'static GlobalData,
}

pub trait ClientEndAfterHandlerImpl {
    async fn client_end(&self, context: Arc<ClientEndContext>) -> Result<(), String>;
}

pub struct ClientEndHandlers {
    handlers: Vec<ClientEndAfterHandle>
}

impl ClientEndHandlers {
    pub fn new(inner: Vec<ClientEndAfterHandle>) -> Self {
        Self {
            handlers: inner
        }
    }

    pub async fn client_end(&self, context: Arc<ClientEndContext>) -> Result<Vec<JoinHandle<Result<(), String>>>, String> {
        let mut handles = Vec::new();
        for handler in self.handlers.iter() {
            let context = context.clone();
            let handler = handler.clone();
            let handler = spawn(async move {
                handler.client_end(context).await
            });
            handles.push(handler);
        }
        Ok(handles)
    }
}