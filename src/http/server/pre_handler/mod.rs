use crate::data::config::entity::runtime_data::GlobalData;
use crate::http::client::client_sender::channel_manager::{
    ChannelSender, ClientSender,
};
use anyhow::Result;
use ntex::http::HeaderMap;

#[macro_use]
mod macros;
pub mod dispatcher;
pub(super) mod model_filter;
pub(super) mod title_catcher;
pub(super) mod user_key_handler;
pub(super) mod userid_handler;
pub(super) mod command;

#[allow(dead_code)]
pub struct ClientJoinContext<'a> {
    pub sender: ClientSender,
    pub user_key: Option<String>,
    pub user_id: Option<i32>,
    pub request_header: &'a HeaderMap,
    pub global_data: &'static GlobalData,
}

pub enum PreHandlerResult {
    Return,
    Pass,
}

pub trait ClientJoinPreHandlerImpl {
    async fn client_join<'a>(
        &'a self,
        context: &mut ClientJoinContext<'a>,
    ) -> Result<PreHandlerResult>;
}