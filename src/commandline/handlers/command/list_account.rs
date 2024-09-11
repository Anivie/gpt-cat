use std::ops::Deref;

use log::info;

use crate::commandline::handlers::dispatcher::{CommandDescription, CommandHandler};
use crate::data::config::entity::runtime_data::GlobalData;

#[derive(Default)]
pub(in crate::commandline::handlers) struct ListAccount;

impl CommandHandler for ListAccount {
    fn description(&self) -> CommandDescription {
        describe! {
            ["list_account" | "la"] help "List all accounts"
        }
    }

    async fn execute(&self, global_data: &GlobalData, _: &Vec<&str>) -> anyhow::Result<()> {
        let account = global_data.account_pool.read();
        let account = account.deref();
        info!("total {} accounts found.", account.len());

        let accounts = sqlx::query!("SELECT * FROM account_list")
            .fetch_all(&global_data.data_base)
            .await?;

        for account in accounts {
            info!("account: {:?}", account);
        }

        Ok(())
    }
}