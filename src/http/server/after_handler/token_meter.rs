use std::sync::{Arc, LazyLock};

use crate::data::http_api::openai::openai_request::MessageUtil;
use crate::http::client::client_sender::channel_manager::ChannelBufferManager;
use crate::http::server::after_handler::{ClientEndAfterHandlerImpl, ClientEndContext};
use color_eyre::owo_colors::OwoColorize;
use log::{error, info};
use tiktoken_rs::{cl100k_base, o200k_base, CoreBPE};

#[derive(Default, Clone)]
pub struct TokenMeterHandler;

static TICK_TOKEN: LazyLock<[CoreBPE; 2]> = LazyLock::new(|| {
    [
        cl100k_base().unwrap(), //for other gpt
        o200k_base().unwrap(),  // for gpt-4o
    ]
});

impl ClientEndAfterHandlerImpl for TokenMeterHandler {
    async fn client_end(&self, context: Arc<ClientEndContext>) -> Result<(), String> {
        let buffer = context.sender.get_buffer();

        let tick_token = match context.sender.request.model.as_str() {
            x if x.contains("4o") || x.contains("o1")  => &TICK_TOKEN[1],
            _ => &TICK_TOKEN[0],
        };

        let user_token = {
            let user_input = context.sender.request.messages.get_all_input();
            let mut user_token = 0;
            for &x in user_input.iter() {
                user_token += tick_token.encode_with_special_tokens(x).len();
            }
            info!(
                "User input: {}, AI output: {}",
                user_input.truecolor(242, 127, 10),
                buffer.purple()
            );
            user_token
        };

        let ai_token = tick_token.encode_with_special_tokens(buffer).len();

        info!("Use of user token: {}, AI token: {}", user_token, ai_token);
        if let Some(price) = context
            .data
            .model_price
            .read()
            .get(&context.sender.request.model)
        {
            info!(
                "model: {}, price: {:?}",
                context.sender.request.model,
                price
            );
            let price = price.clone();

            let insert_id = sqlx::query!(
                "
                    INSERT INTO
                    usage_list (user_id, input_tokens, output_tokens, input_token_price, output_token_price)
                    VALUES
                    ($1, $2, $3, $4, $5)
                ",
                context.user_id,
                user_token as i32,
                ai_token as i32,
                price.input_price,
                price.output_price
            )
                .execute(&context.data.data_base)
                .await
                .map_err(|err| format!("Error when insert usage list: {}", err))?;

            info!(
                "Insert usage last insert id: {:?}, current endpoint: {}",
                insert_id, context.response_data.use_endpoint
            );
        } else {
            error!("Model not found: {}", context.sender.request.model);
        }

        Ok(())
    }
}
