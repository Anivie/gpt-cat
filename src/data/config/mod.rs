//! ## The config file of the server.
//! GPT-Cat will read config from ./config/_.json
//! This App has the following config items:
//! **config.json** The config file of the server, including endpoint url, database path, etc.
//! **model.json** Available model list, including which endpoint have which model
//! **model_price.json** The price of each model

pub mod config_file;
pub mod endpoint;
pub mod model_manager;
pub mod model_price;
pub mod runtime_data;
