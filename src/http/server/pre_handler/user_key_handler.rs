use anyhow::anyhow;

use crate::http::server::pre_handler::{ClientJoinContext, ClientJoinPreHandlerImpl, PreHandlerResult};

#[derive(Default, Clone)]
pub(crate) struct UserKeyHandler;

impl ClientJoinPreHandlerImpl for UserKeyHandler {
    async fn client_join<'a>(
        &'a self,
        context: &mut ClientJoinContext<'a>,
    ) -> anyhow::Result<PreHandlerResult> {
        let auth = if let Some(auth) = context.request_header.get("Authorization")
            && let Ok(auth) = auth.to_str()
        {
            auth[7..].to_string()
        } else {
            return Err(anyhow!("非法请求！您的请求缺少Authorization头部信息"));
        };

        context.user_key.replace(auth);
        Ok(PreHandlerResult::Pass)
    }
}
