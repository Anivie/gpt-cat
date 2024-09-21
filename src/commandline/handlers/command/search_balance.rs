use crate::commandline::handlers::describer::{CommandDescription, CommandHandler};
use crate::data::config::entity::runtime_data::GlobalData;
use cat_macro::describe;
use log::info;

#[derive(Default)]
pub(in crate::commandline::handlers) struct SearchBalance;

impl CommandHandler for SearchBalance {
    fn description(&self) -> CommandDescription {
        describe! {
            ["search_balance" | "sb"] help "Search balance of a user";
            "api_key" => "The api key of the user",
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

        let user = sqlx::query!(
            r#"SELECT * FROM "user" WHERE api_key = $1"#,
            key
        )
        .fetch_one(&global_data.data_base)
        .await?;

        let balance = sqlx::query!(
            r#"SELECT * FROM "user_usage" WHERE user_id = $1"#,
            user.id
        )
        .fetch_one(&global_data.data_base)
        .await?;

        info!("User {:?} has balance: {}.", key, balance.total_purchased);
        Ok(())
    }
}