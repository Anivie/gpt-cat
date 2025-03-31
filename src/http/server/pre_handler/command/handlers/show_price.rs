use anyhow::anyhow;
use rust_decimal::Decimal;
use cat_macro::describe;
use crate::commandline::handlers::describer::CommandDescription;
use crate::http::client::client_sender::channel_manager::ChannelSender;
use crate::http::server::pre_handler::{ClientJoinContext, PreHandlerResult};
use crate::http::server::pre_handler::command::handlers::CommandHandler;

#[derive(Default)]
pub struct ShowPriceHandler;

impl CommandHandler for ShowPriceHandler {
    fn description(&self) -> CommandDescription {
        describe! {
            ["price"] help "展示模型的价格"
            example "`/price 4o` -> Show the price for all model that name contains `4o`.";
            "model_name" => "The name of the model you want to check the price for.",
        }
    }

    async fn execute(&self, context: &mut ClientJoinContext<'_>, args: &Vec<&str>) -> anyhow::Result<PreHandlerResult> {
        let model_name = args.get(0).ok_or(anyhow!("Missing model name"))?;
        let model_name = model_name.to_lowercase();

        let mut price_message = String::from("###  💰模型价格\n");

        let mut is_empty = true;
        context.global_data.model_price.read().iter().for_each(|(model, price)| {
            if model.contains(&model_name) {
                if is_empty {
                    is_empty = false;
                    price_message.push_str("| 模型名称 | 输入价格(元/千token) | 输出价格(元/千token) |\n");
                    price_message.push_str("| --- | --- | --- |\n");
                }
                //当前的模型价格是元每个token，把它转换为元每千token
                let input_price_per_1000_tokens = price.input_price.saturating_mul(Decimal::new(1000, 0));
                let output_price_per_1000_tokens = price.output_price.saturating_mul(Decimal::new(1000, 0));
                price_message.push_str(&format!(
                    "| {} | {:} | {:} |\n",
                    model,
                    input_price_per_1000_tokens,
                    output_price_per_1000_tokens
                ));
            }
        });

        if is_empty {
            price_message.push_str("|  |  |  |\n");
            price_message.push_str("没有找到符合条件的模型价格信息。\n");
        }

        context.sender.send_text(&price_message, false).await?;

        Ok(PreHandlerResult::Return)
    }
}