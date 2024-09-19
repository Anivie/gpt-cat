use crate::commandline::handlers::describer::CommandDescription;
use crate::data::database::entity::user_command::{DataBasePrivateCommand, DataBasePublicCommand};
use crate::data::http_api::openai::openai_request::Message;
use crate::describe;
use crate::http::server::pre_handler::command::handlers::CommandHandler;
use crate::http::server::pre_handler::{ClientJoinContext, PreHandlerResult};
use anyhow::anyhow;
use anyhow::Result;
use log::error;
use rayon::prelude::*;
use std::ops::Deref;

#[derive(Default)]
pub struct TemplateHandler;

impl CommandHandler for TemplateHandler {
    fn description(&self) -> CommandDescription {
        describe! {
            ["template" | "t"] help "A template command.",
            "template name" => "The name of the template you want to use.",
        }
    }

    async fn execute(&self, context: &mut ClientJoinContext<'_>, args: &Vec<&str>) -> Result<PreHandlerResult> {
        let name = args.get(0).ok_or(anyhow!("Missing template name"))?;
        let public_command: Result<DataBasePublicCommand, sqlx::Error> = sqlx::query_as!(
                        DataBasePublicCommand,
                        "SELECT * FROM public_command WHERE command = $1 LIMIT 1",
                        name
                    )
            .fetch_one(&context.global_data.data_base)
            .await;

        let prompt_messages = if let Err(sqlx::Error::RowNotFound) = public_command {
            let private_command: Result<DataBasePrivateCommand, sqlx::Error> = sqlx::query_as!(
                            DataBasePrivateCommand,
                            "SELECT * FROM private_command WHERE user_id = $1 AND command = $2 LIMIT 1",
                            context.user_id,
                            name
                        )
                .fetch_one(&context.global_data.data_base)
                .await;

            if let Err(sqlx::Error::RowNotFound) = private_command {
                return Err(anyhow!("Template not found!"));
            }

            let private_command = private_command.map_err(|e| {
                error!("Error when fetching command: {:?}", e);
                anyhow!("Error when fetching command!")
            })?;

            parse_template(context, private_command.prompt.as_str())?
        }else {
            let public_command = public_command.map_err(|e| {
                error!("Error when fetching command: {:?}", e);
                anyhow!("Error when fetching command!")
            })?;

            parse_template(context, public_command.prompt.as_str())?
        };

        context.sender.request.messages = prompt_messages;
        Ok(PreHandlerResult::Pass)
    }
}

fn parse_template(context: &mut ClientJoinContext, command: &str) -> Result<Vec<Message>> {
    let mut prompt_messages = serde_json::from_str::<Vec<Message>>(command)?;
    let message = context
        .sender
        .request
        .messages
        .clone()
        .into_par_iter()
        .filter(|x| !x.content.starts_with("/t") && !x.content.starts_with("/template"))
        .collect::<Vec<_>>();
    prompt_messages.extend(message);
    Ok(prompt_messages)
}