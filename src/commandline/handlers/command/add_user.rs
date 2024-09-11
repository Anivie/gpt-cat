use log::info;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::commandline::handlers::dispatcher::{CommandDescription, CommandHandler};
use crate::data::config::entity::runtime_data::GlobalData;

#[derive(Default)]
pub(in crate::commandline::handlers) struct AddUser;

impl CommandHandler for AddUser {
    fn description(&self) -> CommandDescription {
        describe! {
            ["add_user" | "au"] help "Add a new user",
            ("api_key") => "The api key of the user, if not provided, a random api key will be generated.";
            ("balance") => "The balance of the user, if not provided, 0 will be set.";
        }
    }

    async fn execute(&self, global_data: &GlobalData, param: &Vec<&str>) -> anyhow::Result<()> {
        let key = if let Some(&first) = param.first() {
            if first.starts_with("sk-") {
                first.to_string()
            }else {
                return Err(anyhow::anyhow!("Invalid api key: key must start with 'sk-'"));
            }
        }else {
            generate_key()
        };

        sqlx::query!(r#"INSERT INTO "user" (api_key) VALUES ($1)"#, key)
            .execute(&global_data.data_base)
            .await?;

        let user = sqlx::query!(r#"SELECT * FROM "user" WHERE api_key = $1"#, key)
            .fetch_one(&global_data.data_base)
            .await?;

        if param.len() > 1 {
            let balance = param[1].parse::<i64>()?;
            sqlx::query!(r#"UPDATE "user_usage" SET total_purchased = $1 WHERE user_id = $2"#, Decimal::from(balance), user.id)
                .execute(&global_data.data_base)
                .await?;
        }

        info!(
            "User {:?} has been added, user: {:?}",
            key, user
        );
        Ok(())
    }
}

fn generate_key() -> String {
    let base = Uuid::new_v4().to_string().replace("-", "");
    let extra = Uuid::new_v4().to_string();
    format!(
        "sk-{}{}{}{}",
        base,
        &extra[0..8],
        &extra[9..13],
        &extra[19..23]
    )
}
