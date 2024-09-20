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
            let mut help_message = String::from("# 🤖 命令帮助\n\n欢迎使用交互式命令！以下是一些可用的命令以及如何使用它们：\n\n## 📢 基本命令\n\n");
            let handlers = HANDLER.deref();
            for handler in handlers {
                help_message.push_str(&handler.description().help_messages());
            }
            help_message.push_str("### 📚 [help, h]：显示帮助页面\n- **command** _(可选)_：指定命令以获取更详细的帮助(仍在施工)\n\n---\n");
            help_message.push_str("\n希望这份帮助页面能让你快速上手！💡 如果有任何疑问，随时可以输入 `help` 命令获取帮助哦！🚀\n");
            help_message
        });

        let message = context
            .sender
            .request
            .messages
            .iter()
            .filter(|&x| (x.role == "system" || x.role == "user") && x.content.starts_with('/'))
            .map(|x| x.content.clone())
            .last();

        if let Some(message) = message {
            let args: Vec<&str> = message.split_whitespace().collect();
            let command = args[0].trim_start_matches('/');
            info!("User {:?} use command: {}", context.user_id, command);

            if command == "help" || command == "h" {
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