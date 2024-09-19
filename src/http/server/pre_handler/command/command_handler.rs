use crate::http::client::client_sender::channel_manager::ChannelSender;
use crate::http::server::pre_handler::command::handlers::CommandHandler;
use crate::http::server::pre_handler::command::{new_command_handler_dispatcher, CommandHandlerDispatcher};
use crate::http::server::pre_handler::{ClientJoinContext, ClientJoinPreHandlerImpl, PreHandlerResult};
use anyhow::Result;
use std::ops::Deref;
use std::sync::LazyLock;
use hashbrown::HashMap;
use log::info;

#[derive(Default, Clone)]
pub struct CommandJoinPreHandler;

impl ClientJoinPreHandlerImpl for CommandJoinPreHandler {
    async fn client_join<'a>(&'a self, context: &mut ClientJoinContext<'a>) -> Result<PreHandlerResult> {
        static HANDLER: LazyLock<Vec<CommandHandlerDispatcher>> = LazyLock::new(|| new_command_handler_dispatcher());
        static HANDLER_MAP: LazyLock<HashMap<&'static str, &CommandHandlerDispatcher>> = LazyLock::new(|| {
            let mut map = HashMap::new();
            for handler in HANDLER.iter() {
                let description = handler.description();
                for &x in description.name.iter() {
                    map.insert(x, handler);
                }
            }
            map
        });
        static HELP_MESSAGE: LazyLock<String> = LazyLock::new(|| {
            let mut back = HANDLER
                .iter()
                .map(|x| {
                    let description = x.description().help_message();
                    format!("{}\n\n", description)
                })
                .collect::<String>();
            back.pop();
            back.pop();

            back
        });

        let message = context
            .sender
            .request
            .messages
            .iter()
            .filter(|&x| (x.role == "system" || x.role == "user") && x.content.starts_with('/'))
            .map(|x| x.content.clone())
            .next();

        if let Some(message) = message {
            let args: Vec<&str> = message.split_whitespace().collect();
            let command = args[0].trim_start_matches('/');
            info!("User {:?} use command: {}", context.user_id, command);

            if command == "help" {
                context.sender.send_text(HELP_MESSAGE.deref(), true).await?;
                return Ok(PreHandlerResult::Return);
            }

            let args: Vec<&str> = args.iter().skip(1).map(|x| x.trim()).collect();
            let handler = HANDLER_MAP.get(command).ok_or(anyhow::anyhow!("Command not found."))?;

            handler.execute(context, &args).await
        }else {
            Ok(PreHandlerResult::Pass)
        }
    }
}