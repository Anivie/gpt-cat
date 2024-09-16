use std::time::Duration;

use log::info;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Proxy};

use crate::data::config::entity::config_file::Config;
use crate::data::config::entity::endpoint::Endpoint;

pub fn get_client(
    proxy_config: &Option<String>,
    config: &Config,
    endpoint: &Endpoint,
    token: &str,
) -> Client {
    let client = Client::builder()
        .read_timeout(Duration::from_secs(config.request_timeout))
        .default_headers(match endpoint {
            Endpoint::QianWen => qian_wen_chat_header_map(token),
            _ => openai_chat_header_map(token),
        })
        .gzip(true)
        .brotli(true)
        .deflate(true);

    let client = if let Some(proxy_server_name) = proxy_config &&
        !proxy_server_name.is_empty()
    {
        let proxy = config.proxy
            .as_ref()
            .expect("Proxy config is not set.")
            .get(proxy_server_name)
            .expect("Proxy server is not found.");

        let address = format!(
            "{}://{}:{}@{}",
            proxy.scheme, proxy_server_name, proxy.password, proxy.address
        );
        info!("Create client with proxy: {}", proxy_server_name);

        client.proxy(Proxy::all(address).unwrap())
    } else {
        client
    };

    client.build().unwrap()
}

fn openai_chat_header_map(token: &str) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "Authorization",
        HeaderValue::from_str(format!("Bearer {}", token).as_str()).unwrap(),
    );
    header_map.insert(
        "Content-Type",
        HeaderValue::from_str("application/json").unwrap(),
    );

    header_map
}

fn qian_wen_chat_header_map(token: &str) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "Authorization",
        HeaderValue::from_str(format!("Bearer {}", token).as_str()).unwrap(),
    );
    header_map.insert(
        "Content-Type",
        HeaderValue::from_str("application/json").unwrap(),
    );

    header_map
}
