use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use colored::Colorize;
use log::{error, info};
use notify::{PollWatcher, RecursiveMode, Watcher};

use crate::data::config::entity::config_file::Config;
use crate::data::config::entity::model_manager::ModelManager;
use crate::data::config::entity::model_mapping::ModelMapping;
use crate::data::config::entity::model_price::ModelPriceMap;
use crate::data::config::entity::runtime_data::GlobalData;

pub fn enable_config_hot_reload(global_data: &GlobalData) -> anyhow::Result<()> {
    info!("{}", "Start watching config file.".to_string().green());

    let config = notify::Config::default()
        .with_compare_contents(true)
        .with_poll_interval(Duration::from_secs(2));

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = PollWatcher::new(tx, config)?;
    watcher.watch(Path::new("./config/config.json"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("./config/model_price.json"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("./config/model.json"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("./config/model_mapping.json"), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                if let Some(path) = event.paths.first() {
                    if let Some(path) = path.file_name()
                        && let Some(path) = path.to_str()
                    {
                        match path {
                            "config.json" => {
                                info!(
                                    "{}",
                                    format!("Start hot reload config file: {:?}", event).blue()
                                );
                                let config = {
                                    let file = File::open("./config/config.json")
                                        .expect("Unable to open file.");
                                    let config = BufReader::new(file);
                                    let config: Config = serde_json::from_reader(config)
                                        .expect("Unable to read json.");
                                    config
                                };
                                *global_data.config.write() = config;
                                info!("{}", "Hot reload config file success.".green());
                            }
                            "model_price.json" => {
                                info!(
                                    "{}",
                                    format!("Start hot reload model price file: {:?}", event)
                                        .blue()
                                );
                                let config = global_data.config.read();
                                *global_data.model_price.write() = ModelPriceMap::new(&config)?;
                                info!("{}", "Hot reload price file success.".green());
                            }
                            "model.json" => {
                                info!(
                                    "{}",
                                    format!("Start hot reload model file: {:?}", event).blue()
                                );
                                let config = global_data.config.read();
                                *global_data.model_info.write() = ModelManager::new(&config)?;
                                info!("{}", "Hot reload model file success.".green());
                            }
                            "model_mapping.json" => {
                                info!(
                                    "{}",
                                    format!("Start hot reload model mapping file: {:?}", event)
                                        .blue()
                                );
                                let config = global_data.config.read();
                                *global_data.model_mapping.write() = ModelMapping::new(&config)?;
                                info!("{}", "Hot reload model mapping file success.".green());
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }

    Ok(())
}
