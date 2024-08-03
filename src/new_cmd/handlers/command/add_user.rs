use log::info;
use uuid::Uuid;

use crate::data::config::runtime_data::GlobalData;
use crate::new_cmd::handlers::dispatcher::{CommandDescription, CommandHandler};

#[derive(Default)]
pub(in crate::new_cmd::handlers) struct AddUser;

impl CommandHandler for AddUser {
    fn description(&self) -> CommandDescription {
        CommandDescription {
            name: vec!["add_user", "au"],
            help: "Add a new user",
            param: Some(vec![("api_key", false), ("balance", false)]),
            param_description: Some(vec![
                    "The api key of the user, if not provided, a random api key will be generated.",
                    "The balance of the user, if not provided, 0 will be set."
            ]),
        }
    }

    async fn execute(&self, global_data: &GlobalData, _: &Vec<&str>) -> anyhow::Result<()> {
        let key = generate_key();
        let user = sqlx::query!(r#"INSERT INTO "user" (api_key) VALUES ($1)"#,key)
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
