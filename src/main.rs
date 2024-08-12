#![feature(let_chains)]
#![allow(unused_doc_comments)]
#![cfg_attr(debug_assertions, allow(warnings))]

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use axum::Router;
use axum::routing::post;
use axum_server::tls_rustls::RustlsConfig;
use fast_log::consts::LogSize;
use fast_log::plugin::file_split::{KeepType, Rolling, RollingType};
use fast_log::plugin::packer::LZ4Packer;
use log::info;
use parking_lot::lock_api::RwLock;
use rustls::crypto::aws_lc_rs;
use tokio::net::TcpListener;
use tokio::task::spawn_blocking;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

use data::config::entity::model_price::ModelPriceMap;
use data::config::entity::runtime_data::{GlobalData, ServerPipeline};
use crate::data::config::config_helper::get_config;
use crate::data::database::database_manager::connect_to_database_sqlx;
use crate::http::client::util::account_manager::load_account_from_database;
use crate::http::client::util::counter::concurrency_pool::VecSafePool;
use crate::http::server::{get_client_end_handler, get_client_join_handler};
use crate::http::server::web::server::main_chat;
use crate::commandline::hot_reload::enable_config_hot_reload;

mod data;
#[macro_use]
mod http;
mod commandline;

fn enable_logging() {
    let config = fast_log::config::Config::new()
        .level(log::LevelFilter::Info)
        .console()
        .chan_len(Some(100000))
        .file_split(
            "./logs/",
            Rolling::new(RollingType::BySize(LogSize::MB(1))),
            KeepType::All,
            LZ4Packer {},
        );
    fast_log::init(config).unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_eyre::install().unwrap();
    enable_logging();
    aws_lc_rs::default_provider().install_default().expect("Error installing default rustls provider");

    let data: &'static GlobalData = {
        // Load config from file
        let config = get_config().expect("Error loading config");

        // Load model price from file
        let model = ModelPriceMap::default();

        // Connect to database
        let db = connect_to_database_sqlx(&config).await.expect("Error connecting to database");

        // Load account from database
        let account = load_account_from_database(&config, &db).await?;
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

    spawn_blocking(move || {
        commandline::handlers::command_listener::add_cmd_listener(data);
    });

    spawn_blocking(move || {
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

    let (http_address, https_address, http_port, https_port, enable_https) = {
        let config = data.config.read();
        let http_address = IpAddr::from_str(config.http_config.http_address.as_str())?;
        let https_address = IpAddr::from_str(config.http_config.https_address.as_str())?;
        let http_port = config.http_config.http_port;
        let https_port = config.http_config.https_port;
        let enable_https = std::path::Path::new("./ssl").exists();
        (http_address, https_address, http_port, https_port, enable_https)
    };

    if enable_https {
        let (cert_path, key_path) = {
            let config = data.config.read();
            (config.http_config.tls_cert_path.clone(), config.http_config.tls_key_path.clone())
        };

        let service_https = service.clone();
        tokio::spawn(async move {
            let rustls = RustlsConfig::from_pem_file(cert_path.as_str(), key_path.as_str()).await.unwrap();
            info!("HTTPS server listening on: {}:{}", https_address, https_port);
            axum_server::bind_rustls(SocketAddr::from((https_address, https_port)), rustls)
                .serve(service_https)
                .await
                .unwrap();
        });
    }

    info!("HTTP server listening on: {}:{}", http_address, http_port);
    let listener = TcpListener::bind(SocketAddr::from((http_address, http_port)))
        .await?;
    axum::serve(listener, service).await?;

    Ok(())
}
