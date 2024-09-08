use anyhow::anyhow;
use crate::commandline::handlers::dispatcher::{CommandDescription, CommandHandler};
use crate::data::config::entity::runtime_data::GlobalData;
use anyhow::Result;
use strum::IntoEnumIterator;
use crate::data::config::entity::endpoint::Endpoint;

#[derive(Default)]
pub(in crate::commandline::handlers) struct ListModel;

impl CommandHandler for ListModel {
    fn description(&self) -> CommandDescription {
        describe! {
            ["list_model" | "lm"] help "List all endpoints support the model",
            "model name" => "Name of the model to list",
        }
    }

    async fn execute(&self, global_data: &GlobalData, args: &Vec<&str>) -> Result<()> {
        let model_name = args.get(0).ok_or_else(|| anyhow!("Model name is required"))?;
        let guard = global_data
            .model_info
            .read();

        let available: Vec<_> = Endpoint::iter()
            .filter(|endpoint| guard.check_available(endpoint, model_name))
            .collect();

        if available.is_empty() {
            println!("No endpoint support the model.");
        } else {
            println!("The model is available for the following endpoints:");
            for endpoint in available {
                println!("{}", endpoint);
            }
        }

        Ok(())
    }
}