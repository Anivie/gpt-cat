use parking_lot::RwLock;
use reqwest::Client;
use sqlx::Pool;
use sqlx_postgres::Postgres;

use crate::data::config::entity::config_file::Config;
use crate::data::config::entity::endpoint::Endpoint;
use crate::data::config::entity::model_manager::ModelManager;
use crate::data::config::entity::model_price::ModelPriceMap;
use crate::http::client::util::counter::concurrency_pool::SafePool;
use crate::http::client::ResponderDispatcher;
use crate::http::server::after_handler::ClientEndHandlers;
use crate::http::server::pre_handler::ClientJoinHandlers;

/// The visitor of the account, which contains the information of the account.
/// It will be used in the account pool, which is used to store the account information.
/// # Fields
/// - account_id: The id of the account.
/// - endpoint: The endpoint of the account.
/// - endpoint_url: The url of the endpoint.
/// - responder: The responder dispatcher of the account.
/// - client: The client of the account.
pub struct AccountVisitor {
    pub account_id: i32,
    pub endpoint: Endpoint,
    pub endpoint_url: String,
    pub responder: ResponderDispatcher,
    pub client: Client,
}

/// The global data, which contains the data that will be used in the whole server.
/// # Fields
/// - data_base: The database connection.
/// - account_pool: The account pool, which is used to store the account information.
/// - config: The config of the server.
/// - model_price: The model price map, which contains the price of the model.
/// - model_info: The model manager, which contains the model info.
pub struct GlobalData {
    pub data_base: Pool<Postgres>,
    pub account_pool: RwLock<Vec<SafePool<AccountVisitor>>>,
    pub config: RwLock<Config>,
    pub model_price: RwLock<ModelPriceMap>,
    pub model_info: RwLock<ModelManager>,
}

/// The server pipeline, which contains the pre-handler and after-handler of the server.
/// # Fields
/// - pre_handler: The pre-handler of the server.
/// - after_handler: The after-handler of the server.
pub struct ServerPipeline {
    pub pre_handler: ClientJoinHandlers,
    pub after_handler: ClientEndHandlers,
}
