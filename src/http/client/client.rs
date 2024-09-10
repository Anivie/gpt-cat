use std::ops::Deref;

use colored::Colorize;
use log::{error, info};

use crate::data::config::entity::endpoint::Endpoint;
use crate::data::config::entity::runtime_data::{AccountVisitor, GlobalData};
use crate::data::openai_api::openai_request::{MessageLocation, MessageUtil};
use crate::http::client::client_sender::channel_manager::{
    ChannelSender, ClientSender, ResponsiveError,
};
use crate::http::client::specific_responder::{ResponderError, SpecificResponder};
use crate::http::client::util::counter::concurrency_pool::{SafeObject, SafePool, VecGettable};

/// The response data from the responder
#[allow(dead_code)]
pub struct ResponseData {
    pub account_id: i32,
    pub use_endpoint: Endpoint,
}

impl GlobalData {
    /// Try to request the endpoint with the sender
    /// # Arguments
    /// * `sender` - The sender that send the request
    /// # Returns
    /// * `Option<ResponseData>` - The response data from the responder
    /// * `None` - If the request failed
    pub async fn try_request(&self, sender: &mut ClientSender) -> Option<ResponseData> {
        let account_pool = self.account_pool.read();
        let mut account = match Self::get_account(sender, &self, account_pool.deref()).await {
            Ok(ok) => ok,
            Err(err) => {
                sender.append_error(ResponsiveError {
                    component: "上游账户池".to_string(),
                    reason: "获取上游失败".to_string(),
                    message: "无法从账户池中读取上游账户信息信息".to_string(),
                    suggestion: Some(
                        "当前账户池无法响应您的请求，请联系我们或稍候重试。".to_string(),
                    ),
                });
                error!("Error when get account visitor: {}", err);
                return None;
            }
        };
        info!("Use of model: {}", sender.request.model);

        let mut account_count = {
            let guard = self.config.read();
            guard.number_can_retries
        };

        info!(
            "Account with id: {}({}) {}: {:?}",
            account.account_id.to_string().blue(),
            account.endpoint,
            "start with prompt".yellow(),
            sender
                .request
                .messages
                .get_user_input(MessageLocation::LAST)
        );

        loop {
            match account.responder.make_response(sender, *account).await {
                Err(err) => match err {
                    ResponderError::Request(err) => {
                        sender.append_error(ResponsiveError {
                            component: "代理器核心".to_string(),
                            reason: "无法连接到服务器".to_string(),
                            message: format!("请求服务失败：{}", err),
                            suggestion: None,
                        });
                        error!(
                            "Error when make request on {}: {}, try again with count {}.",
                            account.endpoint, err, account_count
                        );
                    }
                    ResponderError::Response(err) => {
                        error!(
                            "Success get message, but error when send to client: {}",
                            err.red()
                        );
                        break Some(ResponseData {
                            account_id: account.account_id,
                            use_endpoint: account.endpoint.clone(),
                        });
                    }
                },
                Ok(_) => break Some(ResponseData {
                    account_id: account.account_id,
                    use_endpoint: account.endpoint.clone(),
                }),
            }

            account_count -= 1;
            if account_count <= 0 {
                sender.append_error(ResponsiveError {
                    component: "代理器核心".to_string(),
                    reason: "请求失败".to_string(),
                    message: "无法发起请求，且自动重试以失败告终".to_string(),
                    suggestion: Some(
                        "多个上游均请求失败请考虑当前上游服务崩溃，请等待一段时间后重试"
                            .to_string(),
                    ),
                });

                if let Err(send_error) = sender.send_error().await {
                    error!("Error when send error message: {}", send_error);
                }

                break Some(ResponseData {
                    account_id: account.account_id,
                    use_endpoint: account.endpoint.clone(),
                });
            }

            account = match Self::get_account(sender, &self, account_pool.deref()).await {
                Ok(ok) => ok,
                Err(err) => {
                    sender.append_error(ResponsiveError {
                        component: "上游账户池".to_string(),
                        reason: "获取上游失败".to_string(),
                        message: "无法从账户池中读取上游账户信息信息".to_string(),
                        suggestion: Some(
                            "当前账户池无法响应您的请求，请联系我们或稍候重试。".to_string(),
                        ),
                    });
                    error!("Error when get account visitor: {}", err);
                    return None;
                }
            }
        }
    }

    async fn get_account<'a>(
        sender: &ClientSender,
        data: &'a GlobalData,
        pool: &'a Vec<SafePool<AccountVisitor>>,
    ) -> Result<SafeObject<'a, &'a AccountVisitor>, String> {
        let mut try_count = 0_u8;
        let model_info = data.model_info.read();

        loop {
            if let Some(account) = pool.get_safe_object().await {
                if model_info.check_available(&account.endpoint, &sender.request.model) {
                    return Ok(account);
                }
            }

            if try_count >= 30 {
                return Err("Can't find available account!".to_string());
            }

            try_count += 1;
        }
    }
}
