use sqlx::Pool;
use sqlx_postgres::Postgres;

use crate::data::config::entity::config_file::Config;
use crate::data::config::entity::endpoint::Endpoint;
use crate::data::config::entity::runtime_data::AccountVisitor;
use crate::data::database::entity::data_base_account::DataBaseAccount;
use crate::http::client::util::get_reqwest_client::get_client;

pub async fn load_account_from_database(
    config: &Config,
    db: &Pool<Postgres>,
) -> anyhow::Result<Vec<AccountVisitor>> {
    let row: Vec<DataBaseAccount> = sqlx::query_as!(
        DataBaseAccount,
        "SELECT * from account_list WHERE is_disabled = FALSE"
    )
        .fetch_all(db)
        .await?;

    let back = row.into_iter()
        .map(|account| {
            let endpoint = Endpoint::from_str(&account.endpoint);
            let client = get_client(&account.use_proxy, &config, &endpoint, &account.password);
            AccountVisitor {
                endpoint: endpoint.clone(),

                account_id: account.id,
                endpoint_url: endpoint.to_url(config),

                responder: endpoint.specific_responder_dispatcher(),

                client,
            }
        })
        .collect::<Vec<AccountVisitor>>();

    Ok(back)
}
