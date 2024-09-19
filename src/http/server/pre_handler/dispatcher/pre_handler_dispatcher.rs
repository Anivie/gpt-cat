use crate::http::client::client_sender::channel_manager::{ChannelSender, ResponsiveError};
use crate::http::server::pre_handler::{ClientJoinContext, ClientJoinPreHandlerImpl, PreHandlerResult};
use crate::http::server::ClientJoinPreHandler;
use log::error;

pub struct ClientJoinHandlers {
    handlers: Vec<ClientJoinPreHandler>,
}

impl ClientJoinHandlers {
    pub fn new(inner: Vec<ClientJoinPreHandler>) -> Self {
        Self { handlers: inner }
    }

    pub async fn client_join<'a>(
        &'a self,
        mut context: ClientJoinContext<'a>,
    ) -> ClientJoinContext<'a> {
        for handler in self.handlers.iter() {
            match handler.client_join(&mut context).await {
                Ok(PreHandlerResult::Return) => {
                    context.sender.stopped = true;
                    break;
                }
                Err(error) => {
                    context.sender.append_error(ResponsiveError {
                        component: "预处理器".to_string(),
                        reason: "阻止了您的会话".to_string(),
                        message: error.to_string(),
                        suggestion: None,
                    });
                    error!("ClientJoinHandlers: {}", error.to_string());
                    break;
                }
                Ok(PreHandlerResult::Pass) => {}
            }
        }

        context
    }
}
