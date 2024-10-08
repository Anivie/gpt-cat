use crate::commandline::handlers::describer::CommandDescription;
use crate::data::database::entity::user_usage::UserUsage;
use crate::http::client::client_sender::channel_manager::ChannelSender;
use crate::http::server::pre_handler::command::handlers::CommandHandler;
use crate::http::server::pre_handler::{ClientJoinContext, PreHandlerResult};
use anyhow::anyhow;
use cat_macro::describe;

#[derive(Default)]
pub struct BalanceInquiryHandler;

impl CommandHandler for BalanceInquiryHandler {
    fn description(&self) -> CommandDescription {
        describe! {
            ["balance_inquiry" | "bi"] help "获取您当前的可用余额。"
        }
    }

    async fn execute(&self, context: &mut ClientJoinContext<'_>, _: &Vec<&str>) -> anyhow::Result<PreHandlerResult> {
        let user = context
            .user_id
            .ok_or(anyhow!("Could not found the id for you, please check your api-key and try again."))?;

        let usage: UserUsage = sqlx::query_as!(
            UserUsage,
            r#"
            SELECT * FROM user_usage WHERE user_id = $1 LIMIT 1
            "#,
            user
        ).fetch_one(&context.global_data.data_base).await?;

        let message = format!("当前可用: {}元.", usage.total_purchased);
        context.sender.send_text(&message, true).await?;

        Ok(PreHandlerResult::Return)
    }
}