use rayon::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::config::config_file::Config;
use crate::data::config::endpoint::Endpoint;
use crate::data::config::runtime_data::AccountVisitor;
use crate::data::database::entities::account_list;
use crate::data::database::entities::prelude::AccountList;
use crate::http::client::util::get_reqwest_client::get_client;

pub async fn load_account_from_database(
    config: &Config,
    db: &DatabaseConnection,
) -> Vec<AccountVisitor> {
    AccountList::find()
        .filter(account_list::Column::IsDisabled.eq(false))
        .all(db)
        .await
        .unwrap()
        .into_par_iter()
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
        .collect::<Vec<AccountVisitor>>()
}
