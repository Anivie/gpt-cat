use crate::data::http_api::openai::openai_request::{MessageLocation, MessageUtil};
use crate::http::server::pre_handler::{ClientJoinContext, ClientJoinPreHandlerImpl, PreHandlerResult};

#[derive(Default, Clone)]
pub(crate) struct TitleCatchHandler;

impl ClientJoinPreHandlerImpl for TitleCatchHandler {
    async fn client_join<'a>(
        &'a self,
        context: &mut ClientJoinContext<'a>,
    ) -> anyhow::Result<PreHandlerResult> {
        if let Some(message) = context
            .sender
            .request
            .messages
            .get_user_input(MessageLocation::LAST)
        {
            if (message.starts_with("请总结上述对话为10个字以内的标题")
                || message.starts_with("使用四到五个字直接返回这句话的简要主题"))
                && context.sender.request.model != "gpt-3.5-turbo"
            {
                context.sender.request.model = "gpt-3.5-turbo".to_string();
            }
        }

        Ok(PreHandlerResult::Pass)
    }
}
