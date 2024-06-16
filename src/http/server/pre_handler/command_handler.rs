use std::ops::Deref;
use std::str::FromStr;
use anyhow::anyhow;
use log::info;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::ActiveValue::Set;
use crate::data::database::entities;
use crate::data::database::entities::prelude::{PrivateCommand, PublicCommand};
use crate::data::database::entities::private_command;
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
            .into_iter()
            .filter(|x| {
                x.content.deref() != "/".to_owned() + &$command.command
            })
            .collect::<Vec<_>>();
        prompt_messages.extend(message);
        $context.sender.request.messages = prompt_messages;
    };
}

impl ClientJoinPreHandlerImpl for CommandHandler {
    async fn client_join<'a>(&'a self, context: &mut ClientJoinContext<'a>) -> anyhow::Result<()> {
        let system_message = {
            let system_message = context
                .sender
                .request
                .messages
                .iter()
                .filter(|x| x.role == "system" && x.content.starts_with('/'))
                .collect::<Vec<_>>();

            if system_message.is_empty() {
                return Ok(());
            }

            system_message
                .iter()
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
                    let public_commands = PublicCommand::find()
                        .all(&context.global_data.data_base)
                        .await?;
                    let private_commands = PrivateCommand::find()
                        .all(&context.global_data.data_base)
                        .await?;

                    let mut string = String::from_str("命令 | 描述\n").unwrap();
                    string.push_str("-- | --\n");

                    let map = public_commands
                        .iter()
                        .map(|x| format!("{} | {}", x.command, x.describe));

                    private_commands
                        .iter()
                        .map(|x| format!("{} | {}", x.command, x.describe))
                        .chain(map)
                        .for_each(|x| {
                            string.push_str(&x);
                            string.push('\n');
                        });

                    return Err(anyhow!("Available commands:\n{}", string));
                }
                "custom-command" => {
                    let system_message = system_message.iter()
                        .filter(|&x| x.content.starts_with("/custom-command"))
                        .collect::<Vec<_>>();

                    if context.sender.request.messages.last().unwrap().content.deref() != "/end" {
                        return Err(anyhow!("Please end the custom command first.\nCommand must end with a output with`/end`"));
                    }

                    let split = system_message[0].content[1..].split(" ");
                    let mut split = split.skip(1);
                    let command_name = split.next().ok_or(anyhow!("Error when parse command name"))?.to_string();
                    let command_describe = split.next().ok_or(anyhow!("Error when parse command describe"))?.to_string();

                    let messages = context.sender.request.messages.clone();
                    let messages = messages.into_iter()
                        .filter(|x| !x.content.starts_with("/custom-command"))
                        .filter(|x| x.content.deref() != "/end")
                        .collect::<Vec<_>>();

                    let messages = serde_json::to_string(&messages)?;

                    let mut command = private_command::ActiveModel {
                        user_id: Set(context.user_id),
                        command: Set(command_name.clone()),
                        describe: Set(command_describe),
                        prompt: Set(messages),
                        ..Default::default()
                    };

                    let exist_private_command = PrivateCommand::find()
                        .filter(
                            private_command::Column::UserId.eq(context.user_id)
                                .and(private_command::Column::Command.eq(command_name.clone()))
                        )
                        .one(&context.global_data.data_base)
                        .await?;

                    return if let Some(exist_command) = exist_private_command {
                        command.id = Set(exist_command.id);
                        command.update(&context.global_data.data_base).await?;
                        Err(anyhow!("Command: {} added.", command_name))
                    }else {
                        command.insert(&context.global_data.data_base).await?;
                        Err(anyhow!("Command: {} added.", command_name))
                    }
                }
                command => {
                    let public_command = PublicCommand::find()
                        .filter(entities::public_command::Column::Command.eq(command.to_string()))
                        .one(&context.global_data.data_base)
                        .await?;

                    if let Some(command) = public_command && !command.is_disable {
                        process_message!(command, context, message);
                        info!("User {:?} use public command: {}", context.user_id, command.command);
                    }else {
                        let private_command = PrivateCommand::find()
                            .filter(
                                private_command::Column::Command.eq(command.to_string())
                                    .and(private_command::Column::UserId.eq(context.user_id))
                            )
                            .one(&context.global_data.data_base).await?;
                        if let Some(command) = private_command && !command.is_disable {
                            process_message!(command, context, message);
                            info!("User {:?} use private command: {}", context.user_id, command.command);
                        }else {
                            return Err(anyhow!("Command not found"));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}