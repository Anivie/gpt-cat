use crate::commandline::handlers::describer::{CommandDescription, CommandHandler};
use crate::data::config::entity::runtime_data::GlobalData;
use crate::http::client::util::account_manager::load_account_from_database;
use crate::http::client::util::counter::concurrency_pool::VecSafePool;
use cat_macro::describe;
use log::info;

#[derive(Default)]
pub(in crate::commandline::handlers) struct ManageAccountPool;

impl CommandHandler for ManageAccountPool {
    fn description(&self) -> CommandDescription {
        describe! {
            ["manage_account_pool" | "map"] help "Enable or disable a endpoint in the account pool";
            "endpoint" => "The endpoint to enable or disable",
            "enable" => "Enable or disable the endpoint",
        }
    }

    async fn execute(&self, global_data: &GlobalData, args: &Vec<&str>) -> anyhow::Result<()> {
        let Some(endpoint) = args.first() else {
            return Err(anyhow::anyhow!("Missing endpoint"));
        };

        let enable = if let Some(&enable) = args.get(1) {
            enable.parse::<bool>()?
        } else {
            return Err(anyhow::anyhow!("Missing enable"));
        };

        sqlx::query!(
            r#"UPDATE "account_list" SET is_disabled = $1 WHERE endpoint = $2"#,
            !enable,
            endpoint
        ).execute(&global_data.data_base)
            .await?;

        if enable {
            let visitor = load_account_from_database(&global_data.config.read(), &global_data.data_base).await?;
            let mut pool = global_data.account_pool.write();
            *pool = visitor.to_vec_safe_pool(global_data.config.read().request_concurrency_count);
            info!("Endpoint {} has been enabled, now {} accounts in pool.", endpoint, pool.len());
        }else {
            let mut pool = global_data.account_pool.write();
            pool.retain(|x| x.get_endpoint().to_string().as_str() != *endpoint);
            info!("Endpoint {} has been disabled, now {} accounts in pool.", endpoint, pool.len());
        }

        Ok(())
    }
}