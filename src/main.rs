#![feature(let_chains)]
#![allow(unused_doc_comments)]
#![cfg_attr(debug_assertions, allow(warnings))]

use crate::commandline::hot_reload::enable_config_hot_reload;
use crate::data::config::config_helper::get_config;
use crate::data::database::database_manager::connect_to_database_sqlx;
use crate::http::client::util::account_manager::load_account_from_database;
use crate::http::client::util::counter::concurrency_pool::VecSafePool;
use crate::http::server::web::server::main_chat;
use crate::http::server::{get_client_end_handler, get_client_join_handler};
use anyhow::anyhow;
use data::config::entity::model_price::ModelPriceMap;
use data::config::entity::runtime_data::{GlobalData, ServerPipeline};
use fast_log::consts::LogSize;
use fast_log::plugin::file_split::{KeepType, Rolling, RollingType};
use fast_log::plugin::packer::LZ4Packer;
use log::info;
use ntex::web::middleware::Compress;
use ntex::web::{server, App};
use ntex_cors::Cors;
use parking_lot::lock_api::RwLock;
use rustls::crypto::aws_lc_rs;
use rustls::ServerConfig;
use rustls_pemfile::certs;
use std::fs::File;
use std::io::BufReader;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;
use tokio::task::spawn_blocking;

mod data;
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

#[ntex::main]
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

    let (http_address, https_address, http_port, https_port, enable_https) = {
        let config = data.config.read();
        let http_address = IpAddr::from_str(config.http_config.http_address.as_str())?;
        let https_address = IpAddr::from_str(config.http_config.https_address.as_str())?;
        let http_port = config.http_config.http_port;
        let https_port = config.http_config.https_port;
        let enable_https = Path::new(config.http_config.tls_cert_path.as_str())
            .parent()
            .map_or(false, |p| p.exists());
        (http_address, https_address, http_port, https_port, enable_https)
    };

    info!("HTTP server listening on: {}:{}", http_address, http_port);
    let server = server(move || {
        App::new()
            .service(main_chat)
            .state((data, server_pipeline))
            .wrap(Compress::default())
            .wrap(Cors::new().finish())
    }).bind((http_address, http_port))?;

    let server = if enable_https {
        let (cert_path, key_path) = {
            let config = data.config.read();
            (config.http_config.tls_cert_path.clone(), config.http_config.tls_key_path.clone())
        };

        let key_file = &mut BufReader::new(File::open(key_path)?);
        let key = rustls_pemfile::private_key(key_file)?.ok_or(anyhow!("No private key found"))?;
        let cert_file = &mut BufReader::new(File::open(cert_path)?);
        let cert_chain = certs(cert_file).map(|r| r.unwrap()).collect();
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key)?;

        info!("HTTPS server listening on: {}:{}", https_address, https_port);
        server.bind_rustls(format!("{}:{}", https_address, https_port), config)?
    }else {
        server
    };

    server.run().await?;

    Ok(())
}
