use crate::data::config::runtime_data::GlobalData;
use crate::new_cmd::handlers::dispatcher::CommandHandler;
use log::info;
use uuid::Uuid;


#[derive(Default)]
pub(in crate::new_cmd::handlers) struct AddUser;

impl CommandHandler for AddUser {
    fn name(&self) -> Vec<&str> {
        vec!["adduser", "adu"]
    }

    fn help(&self) -> &str {
        "
            Add a new user,
            default random api key and default balances if not specified.
            [adduser|adu] [api_key] [balances]
        "
    }

    async fn execute(&self, global_data: &GlobalData, _: &Vec<&str>) -> anyhow::Result<()> {
        let key = generate_key();
        let user = sqlx::query!(
            r#"
                INSERT INTO "user" (api_key) VALUES ($1)
            "#,
            key
        )
            .execute(&global_data.data_base)
            .await?;

        info!(
            "User {:?} has been added, id: {:?}",
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
