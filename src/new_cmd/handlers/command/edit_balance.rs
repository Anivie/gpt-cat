use log::info;
use rust_decimal::Decimal;

use crate::data::config::entity::runtime_data::GlobalData;
use crate::new_cmd::handlers::dispatcher::{CommandDescription, CommandHandler};

#[derive(Default)]
pub(in crate::new_cmd::handlers) struct EditUserBalance;

impl CommandHandler for EditUserBalance {
    fn description(&self) -> CommandDescription {
        describe! {
            ["edit_balance" | "eb"] help "Edit balance of a user",
            "api_key" => "The api key of the user",
            "balance" => "The new balance of the user",
        }
    }

    async fn execute(&self, global_data: &GlobalData, args: &Vec<&str>) -> anyhow::Result<()> {
        let key = if let Some(&first) = args.first() {
            if first.starts_with("sk-") {
                first.to_string()
            } else {
                return Err(anyhow::anyhow!(
                    "Invalid api key: key must start with 'sk-'"
                ));
            }
        } else {
            return Err(anyhow::anyhow!("Missing api key"));
        };

        let balance = if let Some(&balance) = args.get(1) {
            balance.parse::<i64>()?
        } else {
            return Err(anyhow::anyhow!("Missing balance"));
        };

        let user = sqlx::query!(
            r#"SELECT * FROM "user" WHERE api_key = $1"#,
            key
        )
        .fetch_one(&global_data.data_base)
        .await?;

        let origin_balance = sqlx::query!(
            r#"SELECT * FROM "user_usage" WHERE user_id = $1"#,
            user.id
        )
        .fetch_one(&global_data.data_base)
        .await?;

        sqlx::query!(
            r#"UPDATE "user_usage" SET total_purchased = $1 WHERE user_id = $2"#,
            Decimal::from(balance),
            user.id
        )
        .execute(&global_data.data_base)
        .await?;

        info!("User {:?} balance has been updated, origin: {}, user: {:?}", key, origin_balance.total_purchased, user);
        Ok(())
    }
}
