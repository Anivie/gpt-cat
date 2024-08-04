use std::fs::File;
use std::io::BufReader;
use crate::data::config::entity::config_file::Config;

pub fn get_config() -> anyhow::Result<Config> {
    let file = File::open("./config/config.json").expect("Unable to open config file.");
    let config = BufReader::new(file);
    let mut config: Config = serde_json::from_reader(config).expect("Unable to read json");

    for (key, value) in std::env::vars() {
        match key.as_str() {
            "DATABASE_URL" => {
                config.database_url = value;
            }
            "NUMBER_CAN_RETRIES" => {
                config.number_can_retries = value.parse()?;
            }
            "REQUEST_CONCURRENCY_COUNT" => {
                config.request_concurrency_count = value.parse()?;
            }
            "ENABLE_HTTPS" => {
                config.http_config.enable_https = value.parse()?;
            }
            "HTTP_ADDRESS" => {
                config.http_config.http_address = value;
            }
            "HTTP_PORT" => {
                config.http_config.http_port = value.parse()?;
            }
            "HTTPS_ADDRESS" => {
                config.http_config.https_address = value;
            }
            "HTTPS_PORT" => {
                config.http_config.https_port = value.parse()?;
            }
            "TLS_CERT_PATH" => {
                config.http_config.tls_cert_path = value.parse()?;
            }
            "TLS_KEY_PATH" => {
                config.http_config.tls_key_path = value.parse()?;
            }
            _ => {}
        }
    }

    Ok(config)
}