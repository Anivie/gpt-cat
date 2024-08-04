use log::info;

use crate::data::config::runtime_data::GlobalData;
use crate::new_cmd::handlers::dispatcher::{CommandDescription, CommandHandler};

#[derive(Default)]
pub(in crate::new_cmd::handlers) struct SearchUser;

impl CommandHandler for SearchUser {
    fn description(&self) -> CommandDescription {
        describe! {
            ["search_user" | "su"] help "Search user by api key",
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

        let usage = sqlx::query!(
            r#"SELECT * FROM "user_usage" WHERE user_id = $1"#,
            user.id
        ).fetch_one(&global_data.data_base)
            .await?;

        info!("User {:?} has been found, usage: {:?}", user, usage);
        Ok(())
    }
}