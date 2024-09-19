use anyhow::{anyhow, Result};

use crate::http::server::pre_handler::{ClientJoinContext, ClientJoinPreHandlerImpl, PreHandlerResult};

#[derive(Default, Clone)]
pub(crate) struct ModelFilterHandler;

impl ClientJoinPreHandlerImpl for ModelFilterHandler {
    async fn client_join<'a>(
        &'a self,
        context: &mut ClientJoinContext<'a>,
    ) -> Result<PreHandlerResult> {
        let model_info = context.global_data.model_info.read();
        let model_price = context.global_data.model_price.read();

        if !model_info.has_model(&context.sender.request.model) {
            return Err(anyhow!(
                "Request model: '{}' could not be found in model pool.",
                &context.sender.request.model
            ));
        }

        if !model_price.contains_key(&context.sender.request.model) {
            return Err(anyhow!(
                "Request model: '{}'s price could not be found in config.",
                &context.sender.request.model
            ));
        }

        drop(model_info);
        Ok(PreHandlerResult::Pass)
    }
}
