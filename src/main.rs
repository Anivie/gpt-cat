#![feature(let_chains)]
#![allow(unused_doc_comments)]
#![cfg_attr(debug_assertions, allow(warnings))]

use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::thread;
use axum::Router;
use axum::routing::post;
use axum_server::tls_rustls::RustlsConfig;

use fast_log::consts::LogSize;
use fast_log::plugin::file_split::RollingType;
use fast_log::plugin::packer::LogPacker;
use log::info;
use parking_lot::lock_api::RwLock;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

use crate::cmd::hot_reload::enable_config_hot_reload;
use crate::data::config::config_file::Config;
use crate::data::config::model_price::ModelPriceMap;
use crate::data::config::runtime_data::{GlobalData, ServerPipeline};
use crate::data::database::database_manager::connect_to_database;
use crate::http::client::util::account_manager::load_account_from_database;
use crate::http::client::util::counter::concurrency_pool::VecSafePool;
use crate::http::server::{get_client_end_handler, get_client_join_handler};
use crate::http::server::web::server::main_chat;

mod data;
#[macro_use]
mod http;
mod cmd;

fn enable_logging() {
    let config = fast_log::config::Config::new()
        .level(log::LevelFilter::Info)
        .console()
        .chan_len(Some(100000))
        .file_split("./logs/",
                    LogSize::MB(1),
                    RollingType::All,
                    LogPacker {}
        );
    fast_log::init(config).unwrap();
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    color_eyre::install().unwrap();
    enable_logging();

    let data: &'static GlobalData = {
        // Load config from file
        let config = {
            let file = File::open("./config/config.json").expect("Unable to open config file.");
            let config = BufReader::new(file);
            let config :Config = serde_json::from_reader(config).expect("Unable to read json");
            config
        };

        let model = ModelPriceMap::default();

        // Connect to database
        let db = connect_to_database(&config).await.unwrap();

        // Load account from database
        let account = load_account_from_database(&config, &db).await;
        info!("Loaded {} accounts from database.", account.len());

        let data = GlobalData {
            data_base: db,
            account_pool: RwLock::new(account.to_vec_safe_pool(config.request_concurrency_count)),
            config: RwLock::new(config),
            model_price: RwLock::new(model),
            model_info: Default::default(),
        };

        Box::leak(Box::new(data))
    };

    tokio::spawn(async move {
        cmd::add_cmd_listener(data).await;
    });

    thread::spawn(move || {
        enable_config_hot_reload(data).unwrap();
    });

    let server_pipeline = ServerPipeline {
        pre_handler: get_client_join_handler(),
        after_handler: get_client_end_handler(),
    };
    let server_pipeline: &'static ServerPipeline = Box::leak(Box::new(server_pipeline));

    let app = Router::new()
        .route("/v1/chat/completions", post(main_chat))
        .with_state((data, server_pipeline))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());
    let service = app.into_make_service();

    let service_http = service.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 7117))).await.unwrap();
        axum::serve(listener, service_http).await.unwrap();
    });

    let rustls = RustlsConfig::from_pem_file("./ssl/fullchain.pem", "./ssl/key.pem").await?;
    axum_server::bind_rustls(SocketAddr::from(([0, 0, 0, 0], 11711)), rustls)
        .serve(service)
        .await?;

    Ok(())
}