use std::ops::Deref;

use anyhow::anyhow;
use log::info;
use rayon::prelude::*;
use crate::data::database::entity::user_command::{DataBasePrivateCommand, DataBasePublicCommand};
use crate::data::openai_api::openai_request::Message;
use crate::http::server::pre_handler::{ClientJoinContext, ClientJoinPreHandlerImpl};

#[derive(Default, Clone)]
pub(crate) struct CommandHandler;

macro_rules! process_message {
    ($command:expr, $context:expr, $message:expr) => {
        let mut prompt_messages = serde_json::from_str::<Vec<Message>>($command.prompt.as_str())?;
        let message = $context
            .sender
            .request
            .messages
            .clone()
            .into_par_iter()
            .filter(|x| x.content.deref() != "/".to_owned() + &$command.command)
            .collect::<Vec<_>>();
        prompt_messages.extend(message);
        $context.sender.request.messages = prompt_messages;
    };
}

impl ClientJoinPreHandlerImpl for CommandHandler {
    async fn client_join<'a>(
        &'a self,
        context: &mut ClientJoinContext<'a>,
    ) -> anyhow::Result<Option<String>> {
        let system_message = {
            let system_message = context
                .sender
                .request
                .messages
                .par_iter()
                .filter(|x| x.role == "system" && x.content.starts_with('/'))
                .collect::<Vec<_>>();

            if system_message.is_empty() {
                return Ok(None);
            }

            system_message
                .par_iter()
                .map(|&x| x.clone())
                .collect::<Vec<_>>()
        };

        for message in &system_message {
            let mut message = &message.content[1..];
            if let Some(space) = message.find(" ") {
                message = &message[..space];
            }
            info!("Try to find command: {}", message);
            match message {
                "help" | "h" => {
                    let public_commands: Vec<DataBasePublicCommand> = sqlx::query_as!(DataBasePublicCommand, "SELECT * FROM public_command")
                        .fetch_all(&context.global_data.data_base)
                        .await?;

                    let user_id = context.user_id.ok_or(anyhow!("User id not found"))?;

                    let private_commands: Vec<DataBasePrivateCommand> = sqlx::query_as!(
                        DataBasePrivateCommand,
                        "SELECT * FROM private_command WHERE user_id = $1",
                        user_id
                    )
                        .fetch_all(&context.global_data.data_base)
                        .await?;

                    let map = public_commands
                        .iter()
                        .map(|x| format!("{} | {}", x.command, x.describe));

                    let mut back = private_commands
                        .iter()
                        .map(|x| format!("{} | {}", x.command, x.describe))
                        .chain(map)
                        .reduce(|mut origin: String, new: String| {
                            origin.push_str(format!("{}\n", new).as_str());
                            origin
                        })
                        .unwrap();
                    back.insert_str(0, "命令 | 描述\n-- | --\n");

                    return Ok(Some(back));
                }
                "custom-command" => {
                    let system_message = system_message
                        .par_iter()
                        .filter(|&x| x.content.starts_with("/custom-command"))
                        .collect::<Vec<_>>();

                    if context
                        .sender
                        .request
                        .messages
                        .last()
                        .unwrap()
                        .content
                        .deref()
                        != "/end"
                    {
                        return Err(anyhow!("Please end the custom command first.\nCommand must end with a output with`/end`"));
                    }

                    let split = system_message[0].content[1..].split(" ");
                    let mut split = split.skip(1);
                    let command_name = split
                        .next()
                        .ok_or(anyhow!("Error when parse command name"))?
                        .to_string();
                    let command_describe = split
                        .next()
                        .ok_or(anyhow!("Error when parse command describe"))?
                        .to_string();

                    let messages = context.sender.request.messages.clone();
                    let messages = messages
                        .into_par_iter()
                        .filter(|x| !x.content.starts_with("/custom-command"))
                        .filter(|x| x.content.deref() != "/end")
                        .collect::<Vec<_>>();

                    let messages = serde_json::to_string(&messages)?;

                    sqlx::query!(
                        "
                            INSERT INTO
                            private_command (user_id, command, describe, prompt)
                            VALUES ($1, $2, $3, $4)
                        ",
                        context.user_id,
                        command_name,
                        command_describe,
                        messages
                    )
                        .execute(&context.global_data.data_base)
                        .await?;

                    let exist_private_command = sqlx::query!(
                        "SELECT * FROM private_command WHERE user_id = $1 AND command = $2",
                        context.user_id,
                        command_name
                    )
                        .fetch_one(&context.global_data.data_base)
                        .await;

                    return if let Ok(_) = exist_private_command {
                        sqlx::query!(
                            "
                                UPDATE private_command
                                SET describe = $1, prompt = $2
                                WHERE user_id = $3 AND command = $4
                            ",
                            command_describe,
                            messages,
                            context.user_id,
                            command_name
                        )
                            .execute(&context.global_data.data_base)
                            .await?;
                        Err(anyhow!("Command: {} added.", command_name))
                    } else {
                        sqlx::query!(
                            "
                                INSERT INTO
                                private_command (user_id, command, describe, prompt)
                                VALUES ($1, $2, $3, $4)
                            ",
                            context.user_id,
                            command_name,
                            command_describe,
                            messages
                        )
                            .execute(&context.global_data.data_base)
                            .await?;
                        Err(anyhow!("Command: {} added.", command_name))
                    }
                }
                command => {
                    let public_command: Result<DataBasePublicCommand, sqlx::Error> = sqlx::query_as!(
                        DataBasePublicCommand,
                        "SELECT * FROM public_command WHERE command = $1 LIMIT 1",
                        command
                    )
                        .fetch_one(&context.global_data.data_base)
                        .await;

                    if let Ok(command) = public_command
                        && !command.is_disable
                    {
                        process_message!(command, context, message);
                        info!(
                            "User {:?} use public command: {}",
                            context.user_id, command.command
                        );
                    } else {
                        /*let private_command = PrivateCommand::find()
                            .filter(
                                private_command::Column::Command
                                    .eq(command.to_string())
                                    .and(private_command::Column::UserId.eq(context.user_id)),
                            )
                            .one(&context.global_data.data_base)
                            .await?;*/
                        let private_command: Result<DataBasePrivateCommand, sqlx::Error> = sqlx::query_as!(
                            DataBasePrivateCommand,
                            "SELECT * FROM private_command WHERE user_id = $1 AND command = $2 LIMIT 1",
                            context.user_id,
                            command
                        )
                            .fetch_one(&context.global_data.data_base)
                            .await;

                        if let Ok(command) = private_command
                            && !command.is_disable
                        {
                            process_message!(command, context, message);
                            info!(
                                "User {:?} use private command: {}",
                                context.user_id, command.command
                            );
                        } else {
                            return Err(anyhow!("Command not found"));
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}
