use crate::commandline::handlers::describer::CommandDescription;
use crate::data::database::entity::user_command::{DataBasePrivateCommand, DataBasePublicCommand};
use crate::data::http_api::openai::openai_request::Message;
use crate::describe;
use crate::http::server::pre_handler::command::handlers::CommandHandler;
use crate::http::server::pre_handler::{ClientJoinContext, PreHandlerResult};
use anyhow::anyhow;
use anyhow::Result;
use log::{error, info};

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

        if let Err(sqlx::Error::RowNotFound) = public_command {
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

            parse_template(context, private_command.prompt.as_str())?;
        }else {
            let public_command = public_command.map_err(|e| {
                error!("Error when fetching command: {:?}", e);
                anyhow!("Error when fetching command!")
            })?;

            parse_template(context, public_command.prompt.as_str())?;
        }

        info!("User {:?} used template {}", context.user_id, name);
        Ok(PreHandlerResult::Pass)
    }
}

fn parse_template(context: &mut ClientJoinContext, command: &str) -> Result<()> {
    let index = context.sender.request.messages
        .iter()
        .enumerate()
        .filter(|(_, x)| x.content.starts_with("/t") || x.content.starts_with("/template"))
        .map(|(index, _)| index)
        .next()
        .ok_or(anyhow!("No message to append to"))?;
    context.sender.request.messages.remove(index);

    let mut prompt_messages = serde_json::from_str::<Vec<Message>>(command)
        .map_err(|e| anyhow!("Error when parsing template: {:?}", e))?;

    for _ in 0..prompt_messages.len() {
        let message = prompt_messages.pop().unwrap();
        context.sender.request.messages.insert(0, message);
    }

    Ok(())
}